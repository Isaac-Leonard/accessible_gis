use assert_no_alloc::assert_no_alloc;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample,
};
use fundsp::{
    hacker::{panner, shared, var},
    prelude::{sine, AudioNode},
};
use itertools::{Either, Itertools};
use ndarray::{Array2, Zip};
use optional_struct::{optional_struct, Applyable};

use std::{collections::HashMap, thread::sleep, time::Duration};

/// Generate a sine wave audio signal for a given frequency.
///
/// # Arguments
///
/// * `frequency` - Frequency of the sine wave in Hertz.
///
/// # Returns
///
/// * A Vec<f64> containing the samples of the sine wave.

pub struct AudioHistogram {
    y: Vec<f64>,
    waveform: Waveform,
    // ... other parameters
    settings: HistogramSettings,
}

enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

impl AudioHistogram {
    pub fn new(y: Vec<f64>, settings: HistogramSettings) -> Self {
        AudioHistogram {
            y,
            waveform: Waveform::Sine,
            settings,
        }
    }

    pub fn play(&self) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => self.run::<f32>(&device, &config.into()),
            cpal::SampleFormat::I16 => self.run::<i16>(&device, &config.into()),
            cpal::SampleFormat::U16 => self.run::<u16>(&device, &config.into()),
            _ => panic!("Unsupported format"),
        }
    }

    fn run<T>(&self, device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>,
    {
        // ... generate sound signal based on self.y and other parameters

        let sample_rate = config.sample_rate.0 as f64;
        let channels = config.channels as usize;
        let HistogramSettings {
            duration,
            min_freq,
            max_freq,
        } = self.settings.clone();
        let freq = shared(440.0);
        let c = var(&freq) >> sine();
        let pos = shared(-1.0);
        let mut c = (c | var(&pos)) >> panner();
        c.set_sample_rate(sample_rate);
        c.allocate();

        let mut next_value = move || assert_no_alloc(|| c.get_stereo());

        let err_fn = |err| panic!("an error occurred on stream: {}", err);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut &mut next_value);
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        let _pos_f = -1.0;
        let min = self
            .y
            .iter()
            .cloned()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();
        let max = self
            .y
            .iter()
            .cloned()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();
        let duration_per_sample_ms = Duration::from_millis((duration / self.y.len()) as u64);
        let y_range = max - min;
        let y_range = if y_range == 0.0 { 1.0 } else { y_range };
        let y_len = self.y.len();
        let freq_range = max_freq - min_freq;
        for (x, y) in self.y.iter().copied().enumerate() {
            let freq_f = (y - min) / y_range * freq_range + min_freq;
            let pos_f = x as f64 / (y_len - 1) as f64 * 2.0 - 1.0;
            pos.set_value(pos_f);
            freq.set_value(freq_f);
            sleep(duration_per_sample_ms);
        }
    }
}

pub fn generate_image_histogram(data: Vec<u8>) -> Vec<f64> {
    let mut counts: Vec<f64> = vec![0.0; 255];
    for x in data {
        counts[x as usize] += 1.0;
    }
    counts
}

pub fn play_histogram(counts: Vec<f64>, settings: HistogramSettings) {
    let sonifier = AudioHistogram::new(counts, settings);
    sonifier.play();
}

pub fn play_rasta(
    data: Array2<f64>,
    min: f64,
    max: f64,
    no_data_value: Option<f64>,
    settings: RasterGraphSettings,
) {
    let rasta_graph = RasterGraph::new(data, min, max, no_data_value, settings);
    rasta_graph.play();
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[optional_struct]
pub struct HistogramSettings {
    /// The length the histogram should play for in milliseconds
    pub duration: usize,
    pub min_freq: f64,
    pub max_freq: f64,
}

impl Default for HistogramSettings {
    fn default() -> Self {
        Self {
            duration: 5000,
            min_freq: 440.0,
            max_freq: 880.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[optional_struct]
pub struct RasterGraphSettings {
    /// The length the histogram should play for in milliseconds
    pub row_duration: Duration,
    pub min_freq: f64,
    pub max_freq: f64,
    pub rows: usize,
    pub cols: usize,
    pub classified: bool,
}

fn count_categories(data: &Array2<f64>, no_data_value: Option<f64>) -> Vec<f64> {
    let to_unique = |x: &&f64| x.to_bits();
    let cmp_floats = |a: &&f64, b: &&f64| a.partial_cmp(b).unwrap();
    if let Some(no_data_value) = no_data_value {
        Either::Left(
            data.iter()
                .filter(move |x| **x != no_data_value && x.is_finite()),
        )
    } else {
        Either::Right(data.iter().filter(|x| x.is_finite()))
    }
    .unique_by(to_unique)
    .sorted_by(cmp_floats)
    .copied()
    .collect()
}

impl Default for RasterGraphSettings {
    fn default() -> Self {
        Self {
            row_duration: Duration::from_millis(1000),
            min_freq: 55.0,
            max_freq: 1760.0,
            rows: 10,
            cols: 10,
            classified: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RasterGraph {
    data: Array2<f64>,
    no_data_value: Option<f64>,
    settings: RasterGraphSettings,
}

impl RasterGraph {
    pub fn new(
        data: Array2<f64>,
        min: f64,
        max: f64,
        no_data_value: Option<f64>,
        settings: RasterGraphSettings,
    ) -> Self {
        Self {
            data,
            no_data_value,
            settings,
        }
    }

    pub fn play(&self) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => self.run::<f32>(&device, &config.into()),
            cpal::SampleFormat::F64 => self.run::<f64>(&device, &config.into()),
            cpal::SampleFormat::I16 => self.run::<i16>(&device, &config.into()),
            cpal::SampleFormat::U16 => self.run::<u16>(&device, &config.into()),
            _ => panic!("Unsupported format"),
        }
    }

    fn run<T>(&self, device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>,
    {
        let RasterGraphSettings {
            row_duration,
            min_freq,
            max_freq,
            classified,
            ..
        } = self.settings;
        let (data, min, max) = if classified {
            let categories = count_categories(&self.data, self.no_data_value);
            let data = self.data.map(|val| {
                categories
                    .iter()
                    .enumerate()
                    .find(|category| val == category.1)
                    .map(|x| x.0 as f64)
                    .or(self.no_data_value)
                    .unwrap_or(f64::NAN)
            });
            let min = *data
                .iter()
                .filter(|x| x.is_finite() && Some(**x) != self.no_data_value)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let max = *data
                .iter()
                .filter(|x| x.is_finite() && Some(**x) != self.no_data_value)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let self_with_data = Self {
                data,
                ..self.clone()
            };
            (self_with_data.calculate_max_count(), min, max)
        } else {
            let min = *self
                .data
                .iter()
                .filter(|x| x.is_finite() && Some(**x) != self.no_data_value)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let max = *self
                .data
                .iter()
                .filter(|x| x.is_finite() && Some(**x) != self.no_data_value)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            (self.calculate_mean(), min, max)
        };
        dbg!(min);
        dbg!(max);

        let row_len = data.ncols() as f64;
        let duration_per_sample_ms = row_duration.div_f64(row_len);
        let y_range = max - min;
        let y_range = if y_range == 0.0 { 1.0 } else { y_range };
        let freq_range = max_freq - min_freq;
        // ... generate sound signal based on self.y and other parameters

        let sample_rate = config.sample_rate.0 as f64;
        let channels = config.channels as usize;
        let freq = shared(0.0);
        let c = var(&freq) >> sine();
        let pos = shared(-1.0);
        let mut c = (c | var(&pos)) >> panner();
        c.set_sample_rate(sample_rate);
        c.allocate();

        let mut next_value = move || assert_no_alloc(|| c.get_stereo());

        let err_fn = |err| panic!("an error occurred on stream: {}", err);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut &mut next_value);
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        let _pos_f = -1.0;
        for row in data.rows() {
            for (i, pixel) in row.indexed_iter() {
                let freq_f = if let Some(pixel) = pixel {
                    (*pixel - min) / y_range * freq_range + min_freq
                } else {
                    0.
                };
                let pos_f = i as f64 / (row_len - 1.) * 2.0 - 1.0;
                pos.set_value(pos_f);
                freq.set_value(freq_f);
                sleep(duration_per_sample_ms);
            }
        }
    }

    fn calculate_mean(&self) -> Array2<Option<f64>> {
        let x_scale = self.data.ncols() / self.settings.cols;
        let y_scale = self.data.nrows() / self.settings.rows;
        Zip::from(self.data.exact_chunks((y_scale, x_scale))).map_collect(|chunk| {
            let cleaned = if let Some(ref no_data_value) = self.no_data_value {
                Either::Left(
                    chunk
                        .into_iter()
                        .filter(move |x| x.is_finite() && *x != no_data_value),
                )
            } else {
                Either::Right(chunk.into_iter().filter(|x| x.is_finite()))
            };
            if cleaned.clone().count() == 0 {
                None
            } else {
                Some(cleaned.sum::<f64>() / (y_scale * x_scale) as f64)
            }
        })
    }

    /// Counts each pixel in each cell and returns the most common one for each cell
    /// Note this isn't very clever so if there's 20 different values then 5% being the same in a cell is enough to return it as the most common, first past the post style
    ///If every cell happens to have the same 5% majority then the whole image will be interpreted as that value.
    fn calculate_max_count(&self) -> Array2<Option<f64>> {
        let x_scale = self.data.ncols() / self.settings.cols;
        let y_scale = self.data.nrows() / self.settings.rows;
        if let Some(no_data_value) = &self.no_data_value {
            Zip::from(self.data.exact_chunks((y_scale, x_scale))).map_collect(|chunk| {
                let mut counts = HashMap::<u64, usize>::new();
                for x in chunk
                    .into_iter()
                    .filter(|x| x.is_finite() && *x != no_data_value)
                {
                    counts
                        .entry(x.to_bits())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                counts
                    .into_iter()
                    .sorted_by_key(|x| x.1)
                    .last()
                    .map(|x| f64::from_bits(x.0))
            })
        } else {
            Zip::from(self.data.exact_chunks((y_scale, x_scale))).map_collect(|chunk| {
                let mut counts = HashMap::<u64, usize>::new();
                for x in chunk.into_iter().filter(|x| x.is_finite()) {
                    counts
                        .entry(x.to_bits())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                counts
                    .into_iter()
                    .sorted_by_key(|x| x.1)
                    .last()
                    .map(|x| f64::from_bits(x.0))
            })
        }
    }
}

fn interpolate_nans(arr: &Array2<f64>) -> Array2<f64> {
    let mut result = arr.clone();
    for ((x, y), el) in arr.indexed_iter() {
        if el.is_nan() {
            let mut sum = 0.0;
            let mut count = 0;
            let neighbours = [
                arr.get((x - 1, y + 1)),
                arr.get((x - 1, y)),
                arr.get((x - 1, y - 1)),
                arr.get((x, y - 1)),
                arr.get((x + 1, y - 1)),
                arr.get((x + 1, y)),
                arr.get((x + 1, y + 1)),
                arr.get((x, y + 1)),
            ];
            for neighbour in neighbours {
                if neighbour.copied().is_some_and(f64::is_finite) {
                    count += 1;
                    sum += *neighbour.unwrap();
                }
            }
            *result.get_mut((x, y)).unwrap() = sum / count as f64;
        }
    }
    result
}

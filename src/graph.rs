use assert_no_alloc::assert_no_alloc;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample,
};
use fundsp::{
    hacker::{constant, panner, shared, var},
    prelude::{sine, AudioNode, Panner},
};
use std::{collections::BTreeMap, f64::consts::PI, ops::Range, thread::sleep_ms};

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
    x: Vec<f64>,
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
        let x = vec![0.0; y.len()];
        AudioHistogram {
            x,
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
        let mut freq = shared(440.0);
        let c = var(&freq) >> sine();
        let mut pos = shared(-1.0);
        let mut c = (c | var(&pos)) >> panner();
        c.set_sample_rate(sample_rate as f64);
        c.allocate();

        let mut next_value = move || assert_no_alloc(|| c.get_stereo());

        let err_fn = |err| panic!("an error occurred on stream: {}", err);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut || next_value());
                },
                err_fn,
                None,
            )
            .unwrap();
        eprintln!("before playing");
        stream.play().unwrap();
        eprintln!("After playing");
        let mut pos_f = -1.0;
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
        let duration_per_sample_ms = duration / self.y.len();
        let duration_per_sample_ms = duration_per_sample_ms as u32;
        let y_range = max - min;
        let y_range = if y_range == 0.0 { 1.0 } else { y_range };
        let y_len = self.y.len();
        let freq_range = max_freq - min_freq;
        for (x, y) in self.y.iter().copied().enumerate() {
            let freq_f = (y - min) / y_range * freq_range + min_freq;
            let pos_f = x as f64 / (y_len - 1) as f64 * 2.0 - 1.0;
            dbg!(pos_f);
            pos.set_value(pos_f);
            freq.set_value(freq_f);
            sleep_ms(duration_per_sample_ms);
        }
    }
}

pub fn generate_image_histogram(mut data: Vec<u8>) -> Vec<f64> {
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

pub fn play_rasta(size: (usize, usize), data: Vec<u32>) {
    let rasta_graph = RastaGraph::new(size, data);
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

#[derive(Debug, Clone)]
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

pub struct RastaGraph {
    data: Vec<Vec<u32>>,
}

impl RastaGraph {
    pub fn new((width, height): (usize, usize), data: Vec<u32>) -> Self {
        let data: Vec<Vec<_>> = data
            .chunks(width)
            .map(|r| r.iter().cloned().collect())
            .collect();
        assert_eq!(data.len(), height);
        Self { data }
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
        let duration = 1000.0;
        let min_freq = 55.0;
        let max_freq = 880.0;
        let mut freq = shared(440.0);
        let c = var(&freq) >> sine();
        let mut pos = shared(-1.0);
        let mut c = (c | var(&pos)) >> panner();
        c.set_sample_rate(sample_rate as f64);
        c.allocate();

        let mut next_value = move || assert_no_alloc(|| c.get_stereo());

        let err_fn = |err| panic!("an error occurred on stream: {}", err);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut || next_value());
                },
                err_fn,
                None,
            )
            .unwrap();
        eprintln!("before playing");
        stream.play().unwrap();
        eprintln!("After playing");
        let mut pos_f = -1.0;
        let min = *self
            .data
            .iter()
            .map(|x| x.iter().min().unwrap())
            .min()
            .unwrap() as f64;
        let max = *self
            .data
            .iter()
            .map(|x| x.iter().max().unwrap())
            .max()
            .unwrap() as f64;
        let y_scale = 100;
        let x_scale = 100;
        let y_len = self.data[0].len() / x_scale;
        let duration_per_sample_ms = duration / y_len as f64;
        let duration_per_sample_ms = duration_per_sample_ms as u32;
        let y_range = max - min;
        let y_range = if y_range == 0.0 { 1.0 } else { y_range };
        let freq_range = max_freq - min_freq;
        for row in self.data.chunks(y_scale).map(|x| {
            let mut averages = vec![0u64; x[0].len()];
            for row in x {
                for (i, val) in row.iter().enumerate() {
                    averages[i] += *val as u64;
                }
            }
            averages
                .into_iter()
                .map(|x| (x / y_scale as u64) as u32)
                .collect::<Vec<u32>>()
        }) {
            for (i, pixel) in row
                .chunks(x_scale)
                .map(|x| x.iter().copied().sum::<u32>() / x_scale as u32)
                .enumerate()
            {
                let freq_f = (pixel as f64 - min) / y_range * freq_range + min_freq;
                let pos_f = i as f64 / (y_len - 1) as f64 * 2.0 - 1.0;
                pos.set_value(pos_f);
                freq.set_value(freq_f);
                sleep_ms(duration_per_sample_ms);
            }
        }
    }
}

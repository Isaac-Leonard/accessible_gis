use itertools::{Either, Itertools};
use ndarray::{Array2, Zip};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, thread::sleep, time::Duration};

use crate::audio::low_level::AudioWave;

use super::{low_level::Playable, Waveform};

pub fn play_rasta(vals: Vec<(Array2<f64>, f64, f64, Option<f64>, RasterGraphSettings)>) {
    let rasta_graph = RasterGraph::new(vals);
    rasta_graph.play();
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct RasterGraphSettings {
    /// The length the histogram should play for in milliseconds
    pub row_duration: Duration,
    pub min_freq: f64,
    pub max_freq: f64,
    pub rows: usize,
    pub cols: usize,
    pub classified: bool,
    pub wave: Waveform,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
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
            wave: Waveform::default(),
            min_value: None,
            max_value: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RasterGraph {
    inner: Vec<RasterGraphInner>,
}

#[derive(Clone, Debug)]
struct RasterGraphInner {
    pub data: Array2<f64>,
    pub no_data_value: Option<f64>,
    pub settings: RasterGraphSettings,
}

impl RasterGraphInner {
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

impl RasterGraph {
    pub fn new(vals: Vec<(Array2<f64>, f64, f64, Option<f64>, RasterGraphSettings)>) -> Self {
        Self {
            inner: vals
                .into_iter()
                .map(|(data, _, _, no_data_value, settings)| RasterGraphInner {
                    data,
                    no_data_value,
                    settings,
                })
                .collect(),
        }
    }
}

impl Playable for RasterGraph {
    fn run<T>(&self, device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>,
    {
        let everything = self
            .inner
            .iter()
            .map(|graph| {
                let RasterGraphSettings {
                    row_duration,
                    min_freq,
                    max_freq,
                    classified,
                    wave,
                    rows: _,
                    cols: _,
                    min_value,
                    max_value,
                } = graph.settings;
                let (data, min, max) = if classified {
                    let categories = count_categories(&graph.data, graph.no_data_value);
                    let data = graph.data.map(|val| {
                        categories
                            .iter()
                            .enumerate()
                            .find(|category| val == category.1)
                            .map(|x| x.0 as f64)
                            .or(graph.no_data_value)
                            .unwrap_or(f64::NAN)
                    });
                    let min = match min_value {
                        Some(min) => min,
                        None => *data
                            .iter()
                            .filter(|x| x.is_finite() && Some(**x) != graph.no_data_value)
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap(),
                    };
                    let max = match max_value {
                        Some(min) => min,
                        None => *data
                            .iter()
                            .filter(|x| x.is_finite() && Some(**x) != graph.no_data_value)
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap(),
                    };
                    let self_with_data = RasterGraphInner {
                        data,
                        ..graph.clone()
                    };
                    (self_with_data.calculate_max_count(), min, max)
                } else {
                    let min = *graph
                        .data
                        .iter()
                        .filter(|x| x.is_finite() && Some(**x) != graph.no_data_value)
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    let max = *graph
                        .data
                        .iter()
                        .filter(|x| x.is_finite() && Some(**x) != graph.no_data_value)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    (graph.calculate_mean(), min, max)
                };
                dbg!(min);
                dbg!(max);

                let row_len = data.ncols() as f64;
                let duration_per_sample_ms = row_duration.div_f64(row_len);
                let y_range = max - min;
                let y_range = if y_range == 0.0 { 1.0 } else { y_range };
                let freq_range = max_freq - min_freq;
                // ... generate sound signal based on self.y and other parameters

                let wave = AudioWave::new::<T>(wave, device, config);
                let _pos_f = -1.0;
                (
                    wave,
                    data,
                    (
                        min,
                        min_freq,
                        y_range,
                        freq_range,
                        row_len,
                        duration_per_sample_ms,
                    ),
                )
            })
            .collect::<Vec<_>>();
        let row_count = everything[0].1.nrows();
        let col_count = everything[0].1.ncols();
        let duration_per_sample_ms = everything[0].2 .5;
        for row in 0..row_count {
            for i in 0..col_count {
                for (wave, data, (min, min_freq, y_range, freq_range, row_len, _)) in
                    everything.iter()
                {
                    let row = data.row(row);
                    let pixel = row[i];
                    let freq_f = if let Some(pixel) = pixel {
                        (pixel - min) / y_range * freq_range + min_freq
                    } else {
                        0.
                    };
                    let pos_f = i as f64 / (row_len - 1.) * 2.0 - 1.0;
                    wave.set_position(pos_f);
                    wave.set_freq(freq_f);
                }
                sleep(duration_per_sample_ms);
            }
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

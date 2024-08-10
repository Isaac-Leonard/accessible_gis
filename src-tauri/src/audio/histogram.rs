use super::low_level::{AudioWave, Playable, Waveform};
use std::{thread::sleep, time::Duration};

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

impl AudioHistogram {
    pub fn new(y: Vec<f64>, settings: HistogramSettings, waveform: Waveform) -> Self {
        AudioHistogram {
            y,
            waveform,
            settings,
        }
    }
}

impl Playable for AudioHistogram {
    fn run<T>(&self, device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>,
    {
        // ... generate sound signal based on self.y and other parameters

        let HistogramSettings {
            duration,
            min_freq,
            max_freq,
        } = self.settings.clone();
        let wave = AudioWave::new::<T>(self.waveform, device, config);
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
            wave.set_position(pos_f);
            wave.set_freq(freq_f);
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

pub fn play_histogram(counts: Vec<f64>, settings: HistogramSettings, wave: Waveform) {
    let sonifier = AudioHistogram::new(counts, settings, wave);
    sonifier.play();
}

#[derive(Debug, Clone, PartialEq)]
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

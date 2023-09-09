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

pub struct Sonify {
    x: Vec<f64>,
    y: Vec<f64>,
    waveform: Waveform,
    // ... other parameters
}

enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

impl Sonify {
    pub fn new(y: Vec<f64>) -> Self {
        let x = vec![0.0; y.len()];
        Sonify {
            x,
            y,
            waveform: Waveform::Sine,
            // ... initialize other parameters with default values
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

        // We would need to generate the signal from the provided data
        // Constants based on the requirements
        let duration: f64 = 0.5; // 0.5 seconds
        let amplitude: f64 = 50.0; // Full amplitude
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
        let duration = 5000;
        let duration_per_sample_ms = duration / self.y.len();
        let duration_per_sample_ms = duration_per_sample_ms as u32;
        let y_range = max - min;
        let y_range = if y_range == 0.0 { 1.0 } else { y_range };
        let y_len = self.y.len();
        let min_freq = 55.0;
        let max_freq = 1760.0 * 4.0;
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

pub fn play(counts: Vec<f64>) {
    let sonifier = Sonify::new(counts);
    sonifier.play();
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

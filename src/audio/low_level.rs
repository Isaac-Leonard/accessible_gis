use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, SizedSample};
use cpal::{Stream, StreamConfig};
use fundsp::hacker::*;

pub fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
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

pub trait Playable {
    fn play(&self) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => self.run::<f32>(&device, &config.into()),
            cpal::SampleFormat::F64 => self.run::<f64>(&device, &config.into()),
            cpal::SampleFormat::I16 => self.run::<i16>(&device, &config.into()),
            cpal::SampleFormat::U8 => self.run::<u8>(&device, &config.into()),
            cpal::SampleFormat::U16 => self.run::<u16>(&device, &config.into()),
            _ => panic!("Unsupported format"),
        }
    }

    fn run<T>(&self, device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>;
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Waveform {
    #[default]
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

pub struct AudioWave {
    freq: Shared<f64>,
    pos: Shared<f64>,
    // This is here as the stream cannot be dropped without ending the audio playback
    #[allow(dead_code)]
    stream: Stream,
}

impl AudioWave {
    pub fn new<T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>>(
        wave: Waveform,
        device: &Device,
        config: &StreamConfig,
    ) -> Self {
        match wave {
            Waveform::Sine => Self::inner_new::<T, _>(sine(), device, config),
            Waveform::Triangle => Self::inner_new::<T, _>(triangle(), device, config),
            Waveform::Square => Self::inner_new::<T, _>(square(), device, config),
            Waveform::Sawtooth => Self::inner_new::<T, _>(saw(), device, config),
        }
    }

    fn inner_new<
        S: cpal::Sample + cpal::SizedSample + cpal::FromSample<f64>,
        T: AudioNode<Sample = f64, Inputs = U1, Outputs = U1> + Send + 'static,
    >(
        source: An<T>,
        device: &Device,
        config: &StreamConfig,
    ) -> Self {
        let freq = shared(440.0);
        let c = var(&freq) >> source;
        let pos = shared(-1.0);
        let mut c = (c | var(&pos)) >> panner();
        c.set_sample_rate(config.sample_rate.0 as f64);
        c.allocate();
        let mut next_value = move || assert_no_alloc(|| c.get_stereo());

        let err_fn = |err| panic!("an error occurred on stream: {}", err);

        let channels = config.channels as usize;
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [S], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut &mut next_value);
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();

        Self { freq, pos, stream }
    }

    pub fn set_position(&self, pos: f64) {
        self.pos.set(pos)
    }

    pub fn set_freq(&self, freq: f64) {
        self.freq.set(freq)
    }
}

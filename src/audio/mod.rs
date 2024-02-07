//! Make some noise via cpal.
#![allow(clippy::precedence)]

pub mod graph;

use std::sync::mpsc;
use std::thread;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, Stream};
use fundsp::hacker::*;
use ndarray::Array2;

use graph::{play_histogram, play_rasta, HistogramSettings, RasterGraphSettings};

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Clone, Debug)]
pub enum AudioMessage {
    PlayHistogram(Vec<f64>, HistogramSettings),
    PlayRaster(Array2<f64>, f64, f64, Option<f64>, RasterGraphSettings),
}

pub fn get_audio() -> mpsc::Sender<AudioMessage> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let stream = get_player();
        stream.pause();
        let mut playing = false;
        loop {
            let msg = rx.recv().unwrap();
            if let AudioMessage::PlayHistogram(graph, settings) = msg {
                /*            if playing {
                        stream.pause();
                    } else {
                        stream.play();
                }*/
                play_histogram(graph, settings);
                playing = !playing;
            } else if let AudioMessage::PlayRaster(data, min, max, no_data_value, settings) = msg {
                play_rasta(data, min, max, no_data_value, settings);
            };
        }
    });
    tx
}

fn get_player() -> Stream {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
        _ => panic!("Unsupported format"),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f64>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    //let c = mls();
    //let c = (mls() | dc(400.0) | dc(50.0)) >> resonator();
    //let c = pink();

    // FM synthesis.
    //let f = 110.0;
    //let m = 5.0;
    //let c = oversample(sine_hz(f) * f * m + f >> sine());

    // Pulse wave.
    //let c = lfo(|t| {
    //    let pitch = 220.0;
    //    let duty = lerp11(0.01, 0.99, sin_hz(0.05, t));
    //    (pitch, duty)
    //}) >> pulse();

    //let c = zero() >> pluck(220.0, 0.8, 0.8);
    //let c = dc(110.0) >> dsf_saw_r(0.99);
    //let c = dc(110.0) >> triangle();
    //let c = dc(110.0) >> soft_saw();
    //let c = lfo(|t| xerp11(20.0, 2000.0, sin_hz(0.1, t))) >> dsf_square_r(0.99) >> lowpole_hz(1000.0);
    //let c = dc(110.0) >> square();
    let c = 0.2 * (organ_hz(midi_hz(57.0)) + organ_hz(midi_hz(61.0)) + organ_hz(midi_hz(64.0)));
    //let c = dc(440.0) >> rossler();
    //let c = dc(110.0) >> lorenz();
    //let c = organ_hz(110.1) + organ_hz(54.9);
    //let c = pink() >> hold_hz(440.0, 0.0);

    // Filtered noise tone.
    //let c = (noise() | dc((440.0, 50.0))) >> !resonator() >> resonator();

    // Test ease_noise.
    //let c = lfo(|t| xerp11(50.0, 5000.0, ease_noise(smooth9, 0, t))) >> triangle();

    // Bandpass filtering.
    //let c = c >> (pass() | envelope(|t| xerp11(500.0, 5000.0, sin_hz(0.05, t)))) >> bandpass_q(5.0);
    //let c = c >> (pass() | envelope(|t| (xerp11(500.0, 5000.0, sin_hz(0.05, t)), 0.9))) >> !bandrez() >> bandrez();

    // Waveshaper.
    //let c = c >> shape(Shape::Crush(20.0));

    // Add feedback delay.
    //let c = c >> (pass() & feedback(butterpass_hz(1000.0) >> delay(1.0) * 0.5));

    // Apply Moog filter.
    //let c = (c | lfo(|t| (xerp11(110.0, 11000.0, sin_hz(0.1, t)), 0.6))) >> moog();

    let c = c >> pan(0.0);

    //let c = fundsp::sound::risset_glissando(false);

    // Add chorus.
    let c = c >> (chorus(0, 0.0, 0.01, 0.2) | chorus(1, 0.0, 0.01, 0.2));

    // Add flanger.
    //let c = c
    //    >> (flanger(0.6, 0.005, 0.01, |t| lerp11(0.005, 0.01, sin_hz(0.1, t)))
    //        | flanger(0.6, 0.005, 0.01, |t| lerp11(0.005, 0.01, cos_hz(0.1, t))));

    // Add phaser.
    //let c = c
    //    >> (phaser(0.5, |t| sin_hz(0.1, t) * 0.5 + 0.5)
    //        | phaser(0.5, |t| cos_hz(0.1, t) * 0.5 + 0.5));

    let mut c = c
        >> (declick() | declick())
        >> (dcblock() | dcblock())
        //>> (multipass() & 0.2 * reverb_stereo(10.0, 3.0))
        >> limiter_stereo((1.0, 5.0));
    //let mut c = c * 0.1;

    c.set_sample_rate(sample_rate);
    c.allocate();

    let mut next_value = move || assert_no_alloc(|| c.get_stereo());
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    Ok(stream)
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

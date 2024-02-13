//! Make some noise via cpal.
#![allow(clippy::precedence)]

pub mod graph;
pub mod histogram;
mod low_level;

pub use low_level::Waveform;
use std::sync::mpsc;
use std::thread;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use fundsp::hacker::*;
use ndarray::Array2;

use graph::{play_rasta, RasterGraphSettings};
use histogram::{play_histogram, HistogramSettings};

use low_level::write_data;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Clone, Debug)]
pub enum AudioMessage {
    PlayHistogram(Vec<f64>, HistogramSettings, Waveform),
    PlayRaster(Array2<f64>, f64, f64, Option<f64>, RasterGraphSettings),
}

pub fn get_audio() -> mpsc::Sender<AudioMessage> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut playing = false;
        loop {
            let msg = rx.recv().unwrap();
            if let AudioMessage::PlayHistogram(graph, settings, wave) = msg {
                play_histogram(graph, settings, wave);
                playing = !playing;
            } else if let AudioMessage::PlayRaster(data, min, max, no_data_value, settings) = msg {
                play_rasta(data, min, max, no_data_value, settings);
            };
        }
    });
    tx
}

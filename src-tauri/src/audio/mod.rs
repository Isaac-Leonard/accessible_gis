//! Make some noise via cpal.
#![allow(clippy::precedence)]

pub mod graph;
pub mod histogram;
pub mod low_level;

pub use low_level::Waveform;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use assert_no_alloc::*;
use ndarray::Array2;

use graph::{play_rasta, RasterGraphSettings};
use histogram::{play_histogram, HistogramSettings};

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Clone, Debug)]
pub enum AudioMessage {
    PlayHistogram(Vec<f64>, HistogramSettings, Waveform),
    PlayRaster(Array2<f64>, f64, f64, Option<f64>, RasterGraphSettings),
}

pub fn get_audio() -> mpsc::SyncSender<AudioMessage> {
    let (tx, rx) = mpsc::sync_channel(0);
    // Note we do not save the returned JoinHandle and so create a detached thread.
    thread::spawn(move || audio_thread(rx));
    tx
}

fn audio_thread(rx: Receiver<AudioMessage>) {
    loop {
        let msg = rx
            .recv()
            .expect("The Sender related to the audio thread has been dropped");
        match msg {
            AudioMessage::PlayHistogram(graph, settings, wave) => {
                play_histogram(graph, settings, wave)
            }
            AudioMessage::PlayRaster(data, min, max, no_data_value, settings) => {
                play_rasta(vec![(data, min, max, no_data_value, settings)]);
            }
        };
    }
}

use std::sync::mpsc::SyncSender;

use gdal::raster::StatisticsMinMax;
use itertools::Itertools;
use tauri::State;

use crate::{audio::AudioMessage, gdal_if::read_raster_data, state::AppState};

#[tauri::command]
#[specta::specta]
pub fn play_as_sound(state: AppState, audio: State<SyncSender<AudioMessage>>) {
    state
        .with_current_raster_band(|band| {
            let Ok(StatisticsMinMax { min, max }) = band.band.band.compute_raster_min_max(false)
            else {
                eprint!("Failed to ge min max for raster");
                return;
            };
            let data = read_raster_data(&band.band.band);
            audio
                .send(AudioMessage::PlayRaster(
                    data,
                    min,
                    max,
                    band.band.no_data_value(),
                    Default::default(),
                ))
                .unwrap();
        })
        .expect("Not a raster band");
}

#[tauri::command]
#[specta::specta]
pub fn play_histogram(state: AppState, audio: State<SyncSender<AudioMessage>>) {
    state
        .with_current_raster_band(|band| {
            let Ok(StatisticsMinMax { min, max }) = band.band.band.compute_raster_min_max(false)
            else {
                eprint!("Failed to ge min max for raster");
                return;
            };
            let Ok(histogram) = band.band.band.histogram(min, max, 256, true, false) else {
                eprint!("Failed to get histogram");
                return;
            };
            audio
                .send(AudioMessage::PlayHistogram(
                    histogram.counts().iter().map(|x| (*x) as f64).collect_vec(),
                    Default::default(),
                    Default::default(),
                ))
                .unwrap();
        })
        .expect("Not a raster band");
}

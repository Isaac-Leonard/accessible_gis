use std::{cmp::Ordering, process::Command};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    dataset_collection::NonEmptyDelegatorImpl,
    gdal_if::{read_raster_data, read_raster_data_enum_as},
    geometry::Point,
    state::{settings::AudioSettings, AppState},
    web_socket::{AppMessage, GisMessage, RasterMessage, TouchDevice, VectorMessage},
};

#[tauri::command]
#[specta::specta]
pub fn generate_counts_report(name: String, state: AppState) {
    let pixels = state
        .with_current_raster_band(|band| read_raster_data(&band.band.band))
        .unwrap();
    let counts = pixels.into_iter().counts_by(|x| x.to_le_bytes());
    let total: f64 = counts.values().sum::<usize>() as f64;
    let mut report = counts
        .into_iter()
        .map(|(pixel, occurences)| {
            (
                f64::from_le_bytes(pixel),
                occurences,
                occurences as f64 / total * 100.0,
            )
        })
        .collect_vec();
    report.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut output = csv::Writer::from_path(name).unwrap();
    output
        .write_record(["value", "count", "percentage"])
        .unwrap();

    report.iter().for_each(|(pixel, count, percentage)| {
        output
            .write_record([
                pixel.to_string(),
                count.to_string(),
                format!("{:.2}", percentage),
            ])
            .unwrap();
    })
}

#[tauri::command]
#[specta::specta]
pub fn classify_current_raster(
    dest: String,
    classifications: Vec<Classification>,
    state: AppState,
) {
    let classifications = classifications
        .into_iter()
        .map(Classification::to_calc_string)
        .join("+");
    state.with_current_dataset_mut(|dataset, _| {
        let mut cmd = Command::new("gdal_calc.py");
        cmd.arg("-A")
            .arg(&dataset.dataset.file_name)
            .arg(format!("--outfile={}", dest))
            .arg(format!("--calc=\"{}\"", classifications))
            .arg(format!(
                "--NoDataValue={}",
                dataset
                    .dataset
                    .dataset
                    .rasterband(1)
                    .unwrap()
                    .no_data_value()
                    .unwrap()
            ));
        let output = cmd.output().expect("Failed to classify raster");
        eprint!("{:?}", output);
    });
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct Classification {
    pub min: f64,
    pub max: f64,
    pub target: f64,
}

impl Classification {
    fn to_calc_string(self) -> String {
        format!("{}*(A>{})*(A<={})", self.target, self.min, self.max)
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_display(state: AppState) {
    state.with_lock(|state| state.shared.display_current_raster())
}

#[tauri::command]
#[specta::specta]
pub fn set_current_audio_settings(
    settings: AudioSettings,
    state: AppState,
    device: State<TouchDevice>,
) {
    state
        .with_current_raster_band(|band| {
            band.info.audio_settings = settings.clone();
            device.send(AppMessage::Gis(GisMessage {
                vector: VectorMessage {},
                raster: RasterMessage {
                    min_freq: settings.min_freq,
                    max_freq: settings.max_freq,
                },
            }));
        })
        .expect("Tried to work on non selected raster band");
}

#[tauri::command]
#[specta::specta]
pub fn get_image_pixels(state: AppState) -> Result<Vec<u8>, String> {
    state
        .with_current_raster_band(|band| {
            band.band
                .band()
                .read_band_as::<u8>()
                .expect("Not u8 data")
                .into_shape_and_vec()
                .1
        })
        .ok_or_else(|| "Couldn't read band data".to_owned())
}

#[tauri::command]
#[specta::specta]
pub fn get_point_of_max_value(state: AppState) -> Option<Point> {
    state
        .with_current_raster_band(|band| {
            let data = read_raster_data(&band.band.band);
            let data_iter = data.indexed_iter();
            match band.band.no_data_value() {
                Some(no_data_value) => itertools::Either::Left(data_iter.filter(move |x| {
                    x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan()
                })),
                _ => itertools::Either::Right(data_iter.filter(|x| !x.1.is_nan())),
            }
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| Point::from_2d_index(index))
        })
        .unwrap()
}

#[tauri::command]
#[specta::specta]
pub fn get_point_of_min_value(state: AppState) -> Option<Point> {
    let mut guard = state.data.lock().unwrap();
    guard
        .with_current_raster_band(|band| {
            let data = read_raster_data(&band.band.band);
            let data_iter = data.indexed_iter();
            match band.band.no_data_value() {
                Some(no_data_value) => itertools::Either::Left(data_iter.filter(move |x| {
                    x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan()
                })),
                _ => itertools::Either::Right(data_iter.filter(|x| !x.1.is_nan())),
            }
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| Point::from_2d_index(index))
        })
        .unwrap()
}

pub trait IntoIndex {
    fn to_2d_index(self) -> (usize, usize);
}

impl IntoIndex for Point {
    fn to_2d_index(self) -> (usize, usize) {
        let Point { x, y } = self;
        (y as usize, x as usize)
    }
}

pub trait FromIndex {
    fn from_2d_index(index: (usize, usize)) -> Point;
}

impl FromIndex for Point {
    fn from_2d_index(index: (usize, usize)) -> Point {
        Self {
            x: index.1 as f64,
            y: index.0 as f64,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_value_at_point(point: Point, state: AppState) -> Option<f64> {
    state
        .with_current_raster_band(|band| {
            let val = read_raster_data_enum_as(
                &band.band.band,
                (point.x.round() as isize, point.y.round() as isize),
                (1, 1),
                (1, 1),
                None,
            )?
            .to_f64()[0];
            if band.band.no_data_value().is_some_and(|ndv| val == ndv) {
                None
            } else {
                Some(val)
            }
        })
        .expect("Tried to get raster band and couldn't find it")
}

#[tauri::command]
#[specta::specta]
pub fn get_band_sizes(state: AppState) -> Vec<RasterSize> {
    state.with_lock(|state| {
        state
            .shared
            .datasets
            .iter_mut()
            .map(|wrapped| {
                let dataset = &wrapped.dataset;
                let (width, length) = dataset.dataset.raster_size();
                let bands = dataset.dataset.raster_count();

                RasterSize {
                    width,
                    length,
                    bands,
                }
            })
            .collect()
    })
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct RasterSize {
    width: usize,
    length: usize,
    bands: usize,
}

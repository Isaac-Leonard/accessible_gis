use std::{cmp::Ordering, path::PathBuf, process::Command};

use gdal::vector::LayerAccess;
use itertools::Itertools;
use proj::Transform;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    dataset_collection::NonEmptyDelegatorImpl,
    errors::ErrorDetails,
    gdal_if::{read_raster_data, read_raster_data_enum_as},
    geometry::Point,
    state::{
        AppState,
        gis::combined::{RasterIndex, StatefulLayerEnum},
        settings::AudioSettings,
    },
    web_socket::{AppMessage, TouchDevice},
};

use super::dataset_collection::NonEmptyDelegatorImplExt;

#[tauri::command]
#[specta::specta]
pub fn generate_counts_report(name: PathBuf, state: AppState) {
    let pixels = state
        .with_current_raster_band_fallible(|band| read_raster_data(&band.band.band))
        .unwrap();
    let counts = pixels
        .into_iter()
        // We need to filter out NANs because we sort the results later
        .filter(|x| !x.is_nan())
        .counts_by(|x| x.to_le_bytes());
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

    for (pixel, count, percentage) in report {
        let res = output.write_record([
            pixel.to_string(),
            count.to_string(),
            format!("{:.2}", percentage),
        ]);
        match res {
            Ok(()) => {}
            Err(err) => {
                state.with_lock(|state| {
                    state.errors.push(ErrorDetails::CsvError(err.into()).into())
                });
                return;
            }
        };
    }
}

#[tauri::command]
#[specta::specta]
pub fn classify_current_raster(
    dest: PathBuf,
    classifications: Vec<Classification>,
    state: AppState,
) {
    let classifications = classifications
        .into_iter()
        .map(Classification::into_calc_string)
        .join("+");
    state.with_current_dataset_mut_fallible(|dataset, _| {
        let mut cmd = Command::new("gdal_calc.py");
        cmd.arg("-A")
            .arg(&dataset.dataset.file_name)
            .arg(format!("--outfile={:?}", dest))
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
        let output = cmd
            .output()
            .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        eprint!("{:?}", output);
        Ok(())
    });
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct Classification {
    pub min: f64,
    pub max: f64,
    pub target: f64,
}

impl Classification {
    fn into_calc_string(self) -> String {
        format!("{}*(A>{})*(A<={})", self.target, self.min, self.max)
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_display_raster(raster: Option<RasterIndex>, state: AppState) {
    state.with_project(|project| project.display_current_raster(raster));
}

#[tauri::command]
#[specta::specta]
pub fn set_current_audio_settings(
    settings: AudioSettings,
    state: AppState,
    device: State<TouchDevice>,
) {
    state.with_project_fallible::<(), _>(|project| {
        let Some(_) = project.with_current_raster_band(|band| {
            band.info.audio_settings = settings.clone();
        }) else {
            return Err(ErrorDetails::Other(
                "Tried to work on non selected raster band".to_string(),
            ));
        };
        device.send(AppMessage::Gis(project.get_touch_device_settings()));
        Ok(())
    });
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
    state.with_current_raster_band_fallible(|band| {
        let data = read_raster_data(&band.band.band)?;
        let data_iter = data.indexed_iter();
        match band.band.no_data_value() {
            Some(no_data_value) => itertools::Either::Left(data_iter.filter(move |x| {
                x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan()
            })),
            _ => itertools::Either::Right(data_iter.filter(|x| !x.1.is_nan())),
        }
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| Point::from_2d_index(index))
        .ok_or_else(|| ErrorDetails::Other("Error generating counts report".to_string()))
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_point_of_min_value(state: AppState) -> Option<Point> {
    state.with_current_raster_band_fallible(|band| {
        let data = read_raster_data(&band.band.band)?;
        let data_iter = data.indexed_iter();
        match band.band.no_data_value() {
            Some(no_data_value) => itertools::Either::Left(data_iter.filter(move |x| {
                x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan()
            })),
            _ => itertools::Either::Right(data_iter.filter(|x| !x.1.is_nan())),
        }
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| Point::from_2d_index(index))
        .ok_or_else(|| ErrorDetails::Other("Error generating counts report".to_string()))
    })
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
            .into_f64_vec()[0];
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
    state
        .with_project(|project| {
            project
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
        .unwrap_or_default()
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct RasterSize {
    width: usize,
    length: usize,
    bands: usize,
}

#[tauri::command]
#[specta::specta]
pub fn focus_dataset(state: AppState, device: State<TouchDevice>) {
    state.with_project_fallible(|project| {
        let (srs, min, max) = project
            .with_current_layer_mut(|layer| {
                Ok(match layer {
                    StatefulLayerEnum::Raster(band) => {
                        let Some(srs) = band.band.srs.clone() else {
                            return Err(ErrorDetails::Other(
                                "No srs available for raster band.".to_string(),
                            ));
                        };

                        let Some([ulx, xres, _xskew, uly, _yskew, yres]) = band.band.geo_transform
                        else {
                            return Err(ErrorDetails::Other(
                                "No geo transform for this dataset".to_string(),
                            ));
                        };

                        let lrx = ulx + (band.band.band().x_size() as f64 * xres);
                        let lry = uly + (band.band.band().y_size() as f64 * yres);
                        let min = geo::point! { x: ulx, y: lry };
                        let max = geo::point! { x:  lrx, y: uly };

                        (srs, min, max)
                    }
                    StatefulLayerEnum::Vector(mut layer) => {
                        let Some(srs) = layer.layer.layer.spatial_ref() else {
                            return Err(ErrorDetails::Other(
                                "No spatial reference system set for this dataset".to_string(),
                            ));
                        };
                        let srs = srs
                            .to_proj4()
                            .map_err(|err| ErrorDetails::Other(err.to_string()))?;

                        let extent = layer.layer.layer().get_extent().map_err(|err| {
                    ErrorDetails::Other(format!(
                        "Could not get extent for layer, there may be no geometries, got error: {}",
                        err
                    ))
                })?;

                        let min = geo_types::point! { x: extent.MinX, y: extent.MinY };
                        let max = geo_types::point! { x: extent.MaxX, y: extent.MaxY };
                        (srs, min, max)
                    }
                })
            })
            .unwrap_or_else(|| Err(ErrorDetails::Other("No layer to work on".to_string())))?;
        let min = min
            .transformed_crs_to_crs(&srs, "WGS84")
            .map_err(|err| ErrorDetails::Other(err.to_string()))?;
        let max = max
            .transformed_crs_to_crs(&srs, "WGS84")
            .map_err(|err| ErrorDetails::Other(err.to_string()))?;

        device.send(AppMessage::FocusBox([min.0.x, min.0.y, max.0.x, max.0.y]));
        Ok(())
    });
}

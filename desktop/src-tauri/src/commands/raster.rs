use std::{
    cmp::Ordering,
    fs::{read, write},
    path::PathBuf,
    process::Command,
};

use gdal::spatial_ref::{CoordTransform, SpatialRef};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    errors::ErrorDetails,
    gdal_if::{read_raster_data, read_raster_data_enum_as},
    geometry::Point,
    state::{
        AppState,
        gis::{combined::RasterIndex, raster::AudioTable},
        settings::AudioSettings,
    },
    web_socket::{AppMessage, TouchDevice},
};

use super::dataset_collection::NonEmptyDelegatorImplExt;

#[tauri::command]
#[specta::specta]
pub fn generate_counts_report(name: PathBuf, state: AppState) {
    let pixels = state
        .with_current_raster_band_fallible(|band| read_raster_data(&band.band.band()))
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
        cmd.arg("-A").arg(&dataset.dataset.file_name);
        cmd.arg(format!("--outfile={:?}", dest));
        cmd.arg(format!("--calc=\"{}\"", classifications));
        if let Some(nda) = dataset
            .get_raster(1)
            .and_then(|band| band.band.no_data_value())
        {
            cmd.arg(format!("--NoDataValue={nda}"));
        }
        let output = cmd
            .output()
            .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        eprint!("{output:?}");
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
pub fn set_display_raster(
    raster: Option<RasterIndex>,
    state: AppState,
    touch_device: State<TouchDevice>,
    app: AppHandle,
) {
    state.with_project_fallible(|project| {
        project.display_current_raster(raster);
        let band = project.get_raster_to_display();
        if let Some(mut band) = band {
            touch_device.send(AppMessage::FetchRaster(band.get_info_for_display(&app)?));
        }
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_current_audio_settings(settings: AudioSettings, state: AppState) {
    state.with_current_raster_band(|band| band.info.audio_settings = settings);
}

#[tauri::command]
#[specta::specta]
pub fn get_point_of_max_value(state: AppState) -> Option<Point> {
    state.with_current_raster_band_fallible(|band| {
        let data = read_raster_data(band.band.band())?;
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
        let data = read_raster_data(band.band.band())?;
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
                band.band.band(),
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
        let (srs, bounds) = project
            .with_current_layer_mut(|mut layer| (layer.get_srs(), layer.get_bounds()))
            .ok_or_else(|| ErrorDetails::Other("No layer to work on".to_string()))?;
        let Some(srs) = srs else {
            return Err(ErrorDetails::Other(
                "No srs available for dataset".to_string(),
            ));
        };
        eprintln!("{:?}", srs.to_pretty_wkt());
        eprintln!("{bounds:?}");
        let Some(bounds) = bounds else {
            return Err(ErrorDetails::Other(
                "Could not get bounds for layer".to_string(),
            ));
        };

        // Unwrap is ssafe here as we have hard coded the epsg code which we know is valid.
        let transform = CoordTransform::new(&srs, &SpatialRef::from_epsg(4326).unwrap())
            .map_err(|err| ErrorDetails::Other(err.to_string()))?;

        let bounds = transform
            .transform_bounds(&bounds, 21)
            .map_err(|err| ErrorDetails::Other(err.to_string()))?;
        eprintln!("{bounds:?}");
        // TODO: Not sure if this is correct however I think it is fine for epsg 4326
        // The order of coordinates seems to be getting switched around in the transformations
        let bounds = [bounds[1], bounds[0], bounds[3], bounds[2]];
        device.send(AppMessage::FocusBox(bounds));
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_audio_table(table: Option<AudioTable>, state: AppState) {
    state.with_current_raster_band(|band| band.info.audio_table = table);
}

#[tauri::command]
#[specta::specta]
pub fn save_audio_table(path: PathBuf, state: AppState) {
    state.with_current_raster_band_fallible(|band| {
        write(
            path,
            serde_json::to_string_pretty(&band.info.audio_table)
                .map_err(|err| ErrorDetails::SerdeError(err.to_string()))?,
        )
        .map_err(|err| ErrorDetails::IoError(err.to_string()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn load_audio_table(path: PathBuf, state: AppState) {
    state.with_current_raster_band_fallible(|band| {
        band.info.audio_table = serde_json::from_slice(
            &read(path).map_err(|err| ErrorDetails::IoError(err.to_string()))?,
        )
        .map_err(|err| ErrorDetails::SerdeError(err.to_string()))?;
        Ok(())
    });
}

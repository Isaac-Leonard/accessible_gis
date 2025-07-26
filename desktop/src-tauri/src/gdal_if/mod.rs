mod dataset;
mod errors;
mod extra_implementations;
mod field_schema;
mod fields;
mod layer;
mod local_feature;
pub mod processing;
mod raster;
pub mod vector;

pub use dataset::*;
pub use field_schema::*;
pub use fields::*;
pub use layer::*;
pub use local_feature::*;
pub use raster::*;

use gdal::{Driver, DriverManager, Metadata, vector::Envelope as GdalEnvelope};
use serde::{Deserialize, Serialize};
use std::path::Path;
pub use vector::*;

use crate::errors::ErrorDetails;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, specta::Type)]
pub struct Envelope {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl From<GdalEnvelope> for Envelope {
    fn from(value: GdalEnvelope) -> Self {
        Self {
            min_x: value.MinX,
            max_x: value.MaxX,
            min_y: value.MinY,
            max_y: value.MaxY,
        }
    }
}

pub fn list_drivers() -> Result<Vec<String>, ErrorDetails> {
    let mut drivers = Vec::new();
    for i in 0..DriverManager::count() {
        drivers.push(
            DriverManager::get_driver(i)
                .map_err(|err| ErrorDetails::Other(err.to_string()))?
                .short_name(),
        )
    }
    Ok(drivers)
}

pub fn get_driver_for_file<P: AsRef<Path>>(path: P) -> Option<Driver> {
    (0..DriverManager::count())
        .filter_map(|index| DriverManager::get_driver(index).ok())
        .find(|driver| {
            let meta = driver
                .metadata()
                .find(|meta| meta.key == "gdal.DMD_EXTENSIONS");
            let Some(meta) = meta else { return false };
            let mut extentions = meta.value.split(' ');
            extentions.any(|x| {
                x == path
                    .as_ref()
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
            })
        })
}

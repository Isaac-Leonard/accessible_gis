mod dataset;
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

use gdal::{vector::Envelope as GdalEnvelope, Driver, DriverManager, Metadata};
use serde::{Deserialize, Serialize};
use std::path::Path;
pub use vector::*;

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

pub fn list_drivers() -> Vec<String> {
    let mut drivers = Vec::new();
    for i in 0..DriverManager::count() {
        drivers.push(DriverManager::get_driver(i).unwrap().short_name())
    }
    drivers
}

pub fn get_driver_for_file<P: AsRef<Path>>(path: P) -> Option<Driver> {
    (0..DriverManager::count())
        .map(|index| DriverManager::get_driver(index).unwrap())
        .find(|driver| {
            let meta = driver
                .metadata()
                .into_iter()
                .find(|meta| meta.key == "gdal.DMD_EXTENSIONS");
            let Some(meta) = meta else { return false };
            let mut extentions = meta.value.split(' ');
            extentions.any(|x| x == path.as_ref().extension().unwrap().to_str().unwrap())
        })
}

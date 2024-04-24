use std::path::PathBuf;

use csv::ReaderBuilder;

use crate::{gdal_if::get_driver_for_file, FeatureInfo};

#[tauri::command]
#[specta::specta]
pub fn get_csv(file: String) -> Vec<Vec<String>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .expect("Could not read file");
    reader
        .deserialize::<Vec<String>>()
        .map(|r| match r {
            Ok(record) => record,
            Err(err) => {
                panic!("Invalid record");
            }
        })
        .collect()
}

pub fn save_vector_file(features: FeatureInfo, name: PathBuf, srs: Option<String>) -> String {
    let driver = get_driver_for_file(&name).expect("Could not find driver for file {name}");
    let dataset = driver.create_vector_only(&name);
    "Error".to_owned()
}

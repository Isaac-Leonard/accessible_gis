use std::{ffi::OsStr, path::PathBuf};

use csv::ReaderBuilder;
use tauri::{AppHandle, Manager, path::BaseDirectory};

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
                panic!("Invalid record: {:?}", err);
            }
        })
        .collect()
}

pub fn get_random_temp_path(app: &AppHandle, ext: Option<impl AsRef<OsStr>>) -> PathBuf {
    let mut path = app
        .path()
        .resolve(uuid::Uuid::new_v4().to_string(), BaseDirectory::Temp)
        .unwrap();
    if let Some(ext) = ext {
        path.set_extension(ext);
    }
    path
}

use std::path::PathBuf;

use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
use serde::{Deserialize, Serialize};

use crate::{
    errors::ErrorDetails,
    gdal_if::{OpenDatasetError, WrappedDataset},
    state::{AppState, Screen, gis::dataset::StatefulDataset},
};

#[tauri::command]
#[specta::specta]
pub fn edit_dataset(state: AppState) {
    state.with_lock(|state| {
        let res: Result<(), EditDatasetError> = state
            .with_current_dataset_mut(|dataset, _| {
                dataset.dataset.dataset = Dataset::open_ex(
                    &dataset.dataset.file_name,
                    DatasetOptions {
                        open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
                        ..Default::default()
                    },
                )
                .map_err(|err| {
                    EditDatasetError::OpenError(OpenDatasetError {
                        name: dataset.dataset.file_name.clone(),
                        gdal_error: err.into(),
                    })
                })?;
                dataset.dataset.editable = true;
                Ok(())
            })
            .expect("No dataset selected");
        match res {
            Ok(()) => {}
            Err(err) => state
                .errors
                .push(ErrorDetails::EditDatasetError(err).into()),
        }
    })
}

#[tauri::command]
#[specta::specta]
pub fn create_new_dataset(driver_name: String, file: String, state: AppState) {
    state.with_lock(|state| {
        let mut dataset = match WrappedDataset::new_vector(file, driver_name) {
            Ok(dataset) => dataset,
            Err(err) => {
                state
                    .errors
                    .push(ErrorDetails::DatasetCreationError(err).into());
                return;
            }
        };
        dataset.add_layer().unwrap();
        let dataset = StatefulDataset::new(dataset, state.settings());
        state.shared.datasets.add(dataset);
        state.screen = Screen::Main;
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_dataset_index(index: usize, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.shared.datasets.set_index(index).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn load_file(name: PathBuf, state: AppState) {
    state.with_lock(|state| {
        state.open_dataset(name);
    });
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub enum EditDatasetError {
    OpenError(OpenDatasetError),
}

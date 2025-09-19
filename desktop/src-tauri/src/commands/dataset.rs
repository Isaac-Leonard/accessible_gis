use std::path::PathBuf;

use serde::Serialize;

use crate::{
    errors::ErrorDetails,
    gdal_if::{FlushCacheError, OpenDatasetError, WrappedDataset},
    state::{AppState, Screen, gis::dataset::StatefulDataset},
};

#[tauri::command]
#[specta::specta]
pub fn edit_dataset(state: AppState) {
    state.with_current_dataset_mut_fallible(|dataset, _| dataset.dataset.reopen_as_editable());
}

#[tauri::command]
#[specta::specta]
pub fn create_new_dataset(driver_name: String, file: String, state: AppState) {
    state.with_project_fallible(|project| {
        let mut dataset = WrappedDataset::new_vector(file, driver_name)
            .map_err(|err| ErrorDetails::DatasetCreationError(err))?;
        dataset
            .add_layer()
            .map_err(|err| ErrorDetails::Other(err))?;
        let dataset = StatefulDataset::new(dataset, &project.settings);
        project.datasets.add(dataset);
        Ok(())
    });
    state.with_lock(|state| state.screen = Screen::Main);
}

#[tauri::command]
#[specta::specta]
pub fn set_dataset_index(index: usize, state: AppState) {
    state.with_project(|project| project.datasets.set_index(index));
}

#[tauri::command]
#[specta::specta]
pub fn load_file(name: PathBuf, state: AppState) {
    state.with_lock(|state| {
        state.open_dataset(name);
    });
}

#[tauri::command]
#[specta::specta]
pub fn load_dataset_multi(name: PathBuf, state: AppState) {
    state.with_lock(|state| {
        state.open_dataset_multi(name);
    });
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub enum EditDatasetError {
    OpenError(OpenDatasetError),
    SaveError(FlushCacheError),
}

impl From<EditDatasetError> for ErrorDetails {
    fn from(value: EditDatasetError) -> Self {
        Self::EditDatasetError(value)
    }
}

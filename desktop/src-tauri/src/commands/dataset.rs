use std::path::PathBuf;

use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
use serde::Serialize;

use crate::{
    errors::ErrorDetails,
    gdal_if::{OpenDatasetError, WrappedDataset},
    state::{AppState, Screen, gis::dataset::StatefulDataset},
};

#[tauri::command]
#[specta::specta]
pub fn edit_dataset(state: AppState) {
    state.with_project_fallible(|project| {
        project
            .with_current_dataset_mut(|dataset, _| {
                let editable_dataset = Dataset::open_ex(
                    &dataset.dataset.file_name,
                    DatasetOptions {
                        open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
                        ..Default::default()
                    },
                )
                .map_err(|err| {
                    ErrorDetails::EditDatasetError(EditDatasetError::OpenError(OpenDatasetError {
                        name: dataset.dataset.file_name.clone(),
                        gdal_error: err.into(),
                    }))
                })?;
                dataset.dataset.dataset = editable_dataset;
                dataset.dataset.editable = true;
                Ok(())
            })
            .expect("No dataset selected")
    });
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

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub enum EditDatasetError {
    OpenError(OpenDatasetError),
}

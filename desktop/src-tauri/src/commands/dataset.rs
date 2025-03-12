use gdal::{Dataset, DatasetOptions, GdalOpenFlags};

use crate::{
    gdal_if::WrappedDataset,
    state::{gis::dataset::StatefulDataset, AppState, Screen},
};

#[tauri::command]
#[specta::specta]
pub fn edit_dataset(state: AppState) -> Result<(), String> {
    eprintln!("here");
    state
        .with_current_dataset_mut(|dataset, _| {
            dataset.dataset.dataset = Dataset::open_ex(
                &dataset.dataset.file_name,
                DatasetOptions {
                    open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
                    ..Default::default()
                },
            )
            .inspect_err(|e| eprintln!("Err: {:?}", e))
            .map_err(|_| {
                format!(
                    "Could not open dataset {} in update mode",
                    &dataset.dataset.file_name
                )
            })?;
            dataset.dataset.editable = true;
            Ok(())
        })
        .expect("No dataset selected")
}

#[tauri::command]
#[specta::specta]
pub fn create_new_dataset(
    driver_name: String,
    file: String,
    state: AppState,
) -> Result<(), String> {
    let mut guard = state.data.lock().unwrap();
    let mut dataset = WrappedDataset::new_vector(file, driver_name)?;
    dataset.add_layer()?;
    let dataset = StatefulDataset::new(dataset, guard.settings());
    guard.shared.datasets.add(dataset);
    guard.screen = Screen::Main;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_dataset_index(index: usize, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.shared.datasets.set_index(index).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn load_file(name: String, state: AppState) -> Result<(), String> {
    state.with_lock(|state| state.open_dataset(name).map(|_| ()))
}

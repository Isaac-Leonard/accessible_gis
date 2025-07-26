use std::path::PathBuf;

use crate::state::{AppState, projects::Project};

use super::dataset_collection::NonEmptyDelegatorImpl;

#[tauri::command]
#[specta::specta]
pub fn create_project(path: PathBuf, state: AppState) {
    state.with_lock(|state| match Project::new(path, state.settings()) {
        Ok(project) => state.project = Some(project),
        Err(err) => state.errors.push(err),
    })
}

#[tauri::command]
#[specta::specta]
pub fn load_project(path: PathBuf, state: AppState) {
    state.with_lock(|state| match Project::load(path) {
        Ok(project) => state.project = Some(project),
        Err(err) => state.errors.push(err),
    })
}

#[tauri::command]
#[specta::specta]
pub fn save_project(state: AppState) {
    state.with_project_fallible(|project| {
        for dataset in project.datasets.iter_mut() {
            dataset.dataset.save()?
        }
        project.save()
    });
}

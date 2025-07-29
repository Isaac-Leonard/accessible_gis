use uuid::Uuid;

use crate::{
    gdal_if::list_drivers,
    state::{AppState, Screen},
    ui::{NewDatasetScreenData, ProjectScreen, TouchDeviceState, UiScreen, UiState},
};

#[tauri::command]
#[specta::specta]
pub fn set_screen(screen: Screen, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.screen = screen;
}

#[tauri::command]
#[specta::specta]
pub fn get_app_info(state: AppState) -> UiState {
    state.with_lock(|state| UiState {
        screen: match state.screen {
            Screen::Main => UiScreen::Project(
                state
                    .get_project_screen_info()
                    .map(ProjectScreen::Project)
                    .unwrap_or_default(),
            ),
            Screen::NewDataset => UiScreen::NewDataset(NewDatasetScreenData {
                // Kind of hacky but this function cannot return an error
                drivers: list_drivers()
                    .map_err(|err| state.errors.push(err.into()))
                    .unwrap_or_default(),
            }),
            Screen::Settings => UiScreen::Settings(state.settings().clone()),
            Screen::TouchDevice => UiScreen::TouchDevice(
                state
                    .with_project(|project| TouchDeviceState {
                        use_labels: project.use_labels,
                        announce_leaving: project.announce_leaving,
                        announce_geometry_type: project.announce_geometry_type,
                    })
                    .unwrap_or_default(),
            ),
            Screen::Errors => UiScreen::Errors,
        },
        errors: state.errors.to_vec(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn mark_error_read(id: Uuid, state: AppState) {
    state.with_lock(|state| state.errors.get(id).unwrap().read = true)
}

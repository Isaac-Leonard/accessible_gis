use crate::{
    gdal_if::list_drivers,
    state::{AppState, Screen},
    ui::{NewDatasetScreenData, UiScreen, UiState},
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
            Screen::Main => UiScreen::Layers(state.get_layers_screen()),
            Screen::NewDataset => UiScreen::NewDataset(NewDatasetScreenData {
                drivers: list_drivers(),
            }),
            Screen::Settings => UiScreen::Settings(state.settings().clone()),
            Screen::TouchDevice => UiScreen::TouchDevice,
        },
        errors: state.errors.clone(),
    })
}

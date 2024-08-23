use crate::{
    gdal_if::list_drivers,
    state::{AppState, Screen},
    ui::{NewDatasetScreenData, UiScreen},
};

#[tauri::command]
#[specta::specta]
pub fn set_screen(screen: Screen, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.screen = screen;
}

#[tauri::command]
#[specta::specta]
pub fn get_app_info(state: AppState) -> UiScreen {
    state.with_lock(|state| match state.screen {
        Screen::Main => UiScreen::Layers(state.get_layers_screen()),
        Screen::NewDataset => UiScreen::NewDataset(NewDatasetScreenData {
            drivers: list_drivers(),
        }),
        Screen::Settings => UiScreen::Settings(state.settings.clone()),
    })
}

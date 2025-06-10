use tauri::AppHandle;
use tauri::Manager;

use crate::state::{AppState, Screen, settings::GlobalSettings};

#[tauri::command]
#[specta::specta]
pub fn open_settings(state: AppState) {
    state.with_lock(|state| {
        state.screen = Screen::Settings;
    })
}

#[tauri::command]
#[specta::specta]
pub fn set_settings(settings: GlobalSettings, state: AppState, app: AppHandle) {
    state.with_lock(|state| {
        state.set_settings(settings, app.path());
    })
}

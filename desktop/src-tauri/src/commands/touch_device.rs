use tauri::State;

use crate::{
    state::AppState,
    web_socket::{AppMessage, TouchDevice},
};

#[tauri::command]
#[specta::specta]
pub fn focus_box(bounds: [f64; 4], device: State<TouchDevice>) {
    device.send(AppMessage::FocusBox(bounds));
}

#[tauri::command]
#[specta::specta]
pub fn toggle_labels(state: AppState, device: State<TouchDevice>) {
    state.with_project(|project| {
        project.use_labels = !project.use_labels;
        device.send(AppMessage::Gis(project.get_touch_device_settings()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_announce_leaving(state: AppState, device: State<TouchDevice>) {
    state.with_project(|project| {
        project.announce_leaving = !project.announce_leaving;
        device.send(AppMessage::Gis(project.get_touch_device_settings()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_announce_geometry_types(state: AppState, device: State<TouchDevice>) {
    state.with_project(|project| {
        project.announce_geometry_type = !project.announce_geometry_type;
        device.send(AppMessage::Gis(project.get_touch_device_settings()))
    });
}

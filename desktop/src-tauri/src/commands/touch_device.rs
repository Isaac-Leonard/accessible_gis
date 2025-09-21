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
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.use_labels = !layer.info.touch_device_settings.use_labels;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_announce_leaving(state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.announce_leaving =
            !layer.info.touch_device_settings.announce_leaving;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_announce_geometry_types(state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.announce_geometry_type =
            !layer.info.touch_device_settings.announce_geometry_type;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

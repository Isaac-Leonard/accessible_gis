use tauri::State;

use crate::{
    state::{AppState, colours::CssColour},
    utils::Toggle,
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
        layer
            .info
            .touch_device_settings
            .visual
            .labels
            .enabled
            .toggle();
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_announce_leaving(state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.audio.announce_leaving =
            !layer.info.touch_device_settings.audio.announce_leaving;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_announce_geometry_types(state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer
            .info
            .touch_device_settings
            .audio
            .announce_geometry_type = !layer
            .info
            .touch_device_settings
            .audio
            .announce_geometry_type;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_vector_line_colour(colour: CssColour, state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.visual.vector_line_colour = colour;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_background_colour(colour: CssColour, state: AppState, device: State<TouchDevice>) {
    state.with_project(|project| {
        project.touch_device_settings.background_colour = colour;
        device.send(AppMessage::UpdateGeneralSettings(
            project.touch_device_settings.clone(),
        ))
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_label_colour(colour: CssColour, state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.visual.labels.text_colour = colour;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_label_font(font: String, state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer.info.touch_device_settings.visual.labels.font = font;
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn toggle_label_fill_text(state: AppState, device: State<TouchDevice>) {
    state.with_current_vector_layer(|layer| {
        layer
            .info
            .touch_device_settings
            .visual
            .labels
            .fill_text
            .toggle();
        device.send(AppMessage::UpdateVector(layer.get_touch_device_info()))
    });
}

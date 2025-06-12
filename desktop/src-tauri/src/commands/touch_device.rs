use tauri::State;

use crate::{
    gdal_if::LayerIndex,
    web_socket::{AppMessage, TouchDevice},
};

#[tauri::command]
#[specta::specta]
pub fn focus_box(bounds: [f64; 4], device: State<TouchDevice>) {
    device.send(AppMessage::FocusBox(bounds));
}

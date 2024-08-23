use crate::state::{gis::raster::RenderMethod, settings::AudioIndicator};

/// This file is for commands that return static data such as names for options

#[tauri::command]
#[specta::specta]
pub fn get_render_methods() -> Vec<RenderMethod> {
    RenderMethod::get_variants()
}

#[tauri::command]
#[specta::specta]
pub fn get_audio_indicators() -> Vec<AudioIndicator> {
    AudioIndicator::get_all_options()
}

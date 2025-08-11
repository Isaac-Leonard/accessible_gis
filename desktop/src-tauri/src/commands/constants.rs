use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    audio::Waveform,
    state::{
        configurable_tools::{
            InputTypeDiscriminants, PresetParameterValueDiscriminants,
            ToolOutputActionDiscriptorDiscriminants,
        },
        gis::raster::RenderMethod,
        settings::AudioIndicator,
    },
};

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

#[tauri::command]
#[specta::specta]
pub fn get_wave_forms() -> Vec<Waveform> {
    Waveform::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_tool_input_types() -> Vec<InputTypeDiscriminants> {
    InputTypeDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_tool_preset_input_types() -> Vec<PresetParameterValueDiscriminants> {
    PresetParameterValueDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_tool_output_actions() -> Vec<ToolOutputActionDiscriptorDiscriminants> {
    ToolOutputActionDiscriptorDiscriminants::iter().collect_vec()
}

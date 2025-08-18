use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    audio::Waveform,
    state::{
        gis::raster::RenderMethod,
        settings::AudioIndicator,
        tools::{ToolInputTypeDiscriminants, ToolPresetParameterValueDiscriminants},
        workflows::WorkflowInputValueDescriptorDiscriminants,
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
pub fn get_tool_input_types() -> Vec<ToolInputTypeDiscriminants> {
    ToolInputTypeDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_tool_preset_input_types() -> Vec<ToolPresetParameterValueDiscriminants> {
    ToolPresetParameterValueDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_workflow_input_types() -> Vec<WorkflowInputValueDescriptorDiscriminants> {
    WorkflowInputValueDescriptorDiscriminants::iter().collect_vec()
}

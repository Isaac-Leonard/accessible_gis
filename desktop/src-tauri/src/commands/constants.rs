use std::collections::BTreeMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    audio::Waveform,
    state::{
        AppState,
        colours::{CssColourDiscriminants, NamedColour},
        gis::raster::{AudioTypeDiscriminants, EscSound, RenderMethod},
        settings::AudioIndicator,
        tools::{ToolInputTypeDiscriminants, ToolPresetParameterValueDiscriminants},
    },
};

/// This file is for commands that return static data such as names for options

#[tauri::command]
#[specta::specta]
pub fn get_render_methods() -> Vec<RenderMethod> {
    RenderMethod::iter().collect_vec()
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
pub fn get_audio_types() -> Vec<AudioTypeDiscriminants> {
    AudioTypeDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_esc_sounds(state: AppState) -> BTreeMap<String, Vec<IndexedEscSound>> {
    state
        .default_data
        .esc_sounds
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, sound)| IndexedEscSound { index, sound })
        .into_group_map_by(|sound| sound.sound.category.clone())
        .into_iter()
        .collect::<BTreeMap<_, _>>()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct IndexedEscSound {
    index: usize,
    #[serde(flatten)]
    sound: EscSound,
}

#[tauri::command]
#[specta::specta]
pub fn get_css_colour_types() -> Vec<CssColourDiscriminants> {
    CssColourDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
pub fn get_named_colours() -> Vec<NamedColour> {
    NamedColour::iter().collect_vec()
}

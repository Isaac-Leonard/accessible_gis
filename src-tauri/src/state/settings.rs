use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use super::gis::raster::RenderMethod;

const DEFAULT_SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct GlobalSettings {
    show_towns_by_default: bool,
    show_countries_by_default: bool,
    display_first_raster: bool,
    default_ocr_for_gdal: bool,
    default_rendering_method_for_images: RenderMethod,
    audio: AudioSettings,
}

impl GlobalSettings {
    fn write_to_file(&self) {
        std::fs::write(
            DEFAULT_SETTINGS_FILE_NAME,
            serde_json::to_string_pretty(&self).expect("Could not serialise settings"),
        )
        .expect("Could not save settings")
    }

    pub fn read() -> Self {
        std::fs::read("settings.json")
            .map(|x| serde_json::from_slice(&x).expect("Could not read settings file"))
            .unwrap_or_default()
    }

    pub fn set_display_first_raster(&mut self, display_first_raster: bool) {
        self.display_first_raster = display_first_raster;
        self.write_to_file()
    }

    pub fn set_show_towns_by_default(&mut self, show_towns_by_default: bool) {
        self.show_towns_by_default = show_towns_by_default;
        self.write_to_file()
    }

    pub fn set_show_countries_by_default(&mut self, show_countries_by_default: bool) {
        self.show_countries_by_default = show_countries_by_default;
        self.write_to_file()
    }

    pub fn display_first_raster(&self) -> bool {
        self.display_first_raster
    }

    pub fn show_countries_by_default(&self) -> bool {
        self.show_countries_by_default
    }

    pub fn show_towns_by_default(&self) -> bool {
        self.show_towns_by_default
    }

    pub fn default_ocr_for_gdal(&self) -> bool {
        self.default_ocr_for_gdal
    }

    pub fn set_default_ocr_for_gdal(&mut self, default_ocr_for_gdal: bool) {
        self.default_ocr_for_gdal = default_ocr_for_gdal;
        self.write_to_file()
    }

    pub fn default_rendering_method_for_images(&self) -> RenderMethod {
        self.default_rendering_method_for_images
    }

    pub fn set_default_rendering_method_for_images(
        &mut self,
        default_rendering_method: RenderMethod,
    ) {
        self.default_rendering_method_for_images = default_rendering_method;
        self.write_to_file()
    }

    pub fn get_default_audio(&self) -> &AudioSettings {
        &self.audio
    }

    pub fn set_default_audio(&mut self, audio: AudioSettings) {
        self.audio = audio;
        self.write_to_file()
    }
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            show_towns_by_default: false,
            show_countries_by_default: false,
            display_first_raster: true,
            default_ocr_for_gdal: false,
            default_rendering_method_for_images: RenderMethod::Image,
            audio: AudioSettings::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct AudioSettings {
    pub min_freq: f64,
    pub max_freq: f64,
    pub volume: f64,
    no_data_value_sound: AudioIndicator,
    border_sound: AudioIndicator,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            min_freq: 220.0,
            max_freq: 880.0,
            volume: 1.0,
            no_data_value_sound: AudioIndicator::Different,
            border_sound: AudioIndicator::MinFreq,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type, EnumIter)]
pub enum AudioIndicator {
    Silence,
    MinFreq,
    MaxFreq,
    Verbal,
    Different,
}

impl AudioIndicator {
    pub fn get_all_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}

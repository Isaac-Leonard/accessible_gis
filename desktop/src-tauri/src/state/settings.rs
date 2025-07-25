use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use tauri::{
    Runtime,
    path::{BaseDirectory, PathResolver},
};

use crate::{
    audio::{graph::RasterGraphSettings, histogram::HistogramSettings},
    errors::ErrorDetails,
};

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
    pub fn write_to_file<R: Runtime>(
        &self,
        resolver: &PathResolver<R>,
    ) -> Result<(), ErrorDetails> {
        let app_data_dir = resolver
            .app_data_dir()
            .map_err(|err| ErrorDetails::TauriError(err.to_string()))?;
        if !app_data_dir.exists() {
            std::fs::create_dir(&app_data_dir)
                .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        };
        let dest = resolver
            .resolve(DEFAULT_SETTINGS_FILE_NAME, BaseDirectory::AppData)
            .map_err(|err| ErrorDetails::TauriError(err.to_string()))?;
        let settings_json = serde_json::to_string_pretty(&self)
            .map_err(|err| ErrorDetails::SerdeError(err.to_string()))?;
        std::fs::write(dest, settings_json).map_err(|err| ErrorDetails::IoError(err.to_string()))
    }

    pub fn read<R: Runtime>(resolver: &PathResolver<R>) -> Result<Self, ErrorDetails> {
        let dest = resolver
            .resolve(DEFAULT_SETTINGS_FILE_NAME, BaseDirectory::AppData)
            .map_err(|err| ErrorDetails::TauriError(err.to_string()))?;
        eprintln!("Reading settings from {:?}", dest);
        let json = std::fs::read(dest).map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        serde_json::from_slice(&json).map_err(|err| ErrorDetails::SerdeError(err.to_string()))
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

    pub fn default_rendering_method_for_images(&self) -> RenderMethod {
        self.default_rendering_method_for_images
    }

    pub fn get_default_audio(&self) -> &AudioSettings {
        &self.audio
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
    histogram: HistogramSettings,
    graph: RasterGraphSettings,
}

impl AudioSettings {
    pub fn histogram(&self) -> &HistogramSettings {
        &self.histogram
    }

    pub fn graph(&self) -> &RasterGraphSettings {
        &self.graph
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            min_freq: 220.0,
            max_freq: 880.0,
            volume: 1.0,
            no_data_value_sound: AudioIndicator::Different,
            border_sound: AudioIndicator::MinFreq,
            histogram: HistogramSettings::default(),
            graph: RasterGraphSettings::default(),
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

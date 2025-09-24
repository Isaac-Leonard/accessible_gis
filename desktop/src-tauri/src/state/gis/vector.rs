use std::{
    hash::{DefaultHasher, Hash, Hasher},
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};

use crate::{
    commands::SortOption,
    gdal_if::{Srs, WrappedLayer},
    state::colours::{CssColour, NamedColour},
    web_socket::VectorInfo,
};

use super::shared::SharedInfo;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct StatefulVectorInfo {
    pub shared: SharedInfo,
    pub display: bool,
    #[serde(default)]
    pub desktop_settings: DesktopVectorOptions,
    #[serde(default)]
    pub touch_device_settings: TouchDeviceVectorOptions,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct DesktopVectorOptions {
    /// The index of each user selected feature for each layer of the dataset
    pub selected_feature: Option<usize>,
    /// The name of the field used to identify features
    pub primary_field_name: Option<String>,
    pub sort_features_by: SortOption,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, specta::Type)]
pub struct TouchDeviceVectorOptions {
    pub audio: TouchDeviceAudioVectorOptions,
    pub visual: TouchDeviceVisualVectorOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct TouchDeviceVisualVectorOptions {
    pub vector_line_colour: CssColour,
    #[serde(default = "TouchDeviceVisualVectorOptions ::default_point_radius")]
    pub point_radius: usize,
    #[serde(default = "TouchDeviceVisualVectorOptions::default_line_width")]
    pub line_width: usize,
    #[serde(default)]
    pub labels: TouchDeviceLabelOptions,
}

impl TouchDeviceVisualVectorOptions {
    pub const fn default_point_radius() -> usize {
        5
    }

    pub const fn default_line_width() -> usize {
        2
    }
}

impl Default for TouchDeviceVisualVectorOptions {
    fn default() -> Self {
        Self {
            vector_line_colour: CssColour::Named(NamedColour::White),
            point_radius: Self::default_point_radius(),
            line_width: Self::default_line_width(),
            labels: TouchDeviceLabelOptions::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct TouchDeviceLabelOptions {
    pub enabled: bool,
    pub text_colour: CssColour,
    pub font: String,
    #[serde(default = "TouchDeviceLabelOptions::default_line_width")]
    pub line_width: usize,
    pub fill_text: bool,
    pub prefered_label_field: Option<String>,
}

impl TouchDeviceLabelOptions {
    pub const fn default_line_width() -> usize {
        2
    }
}

impl Default for TouchDeviceLabelOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            prefered_label_field: None,
            text_colour: CssColour::Named(NamedColour::LightYellow),
            font: "20px sans-serif".to_string(),
            line_width: Self::default_line_width(),
            fill_text: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct TouchDeviceAudioVectorOptions {
    pub prefered_label_field: Option<String>,
    pub announce_leaving: bool,
    pub announce_geometry_type: bool,
    // In kilometres
    #[serde(default = "TouchDeviceAudioVectorOptions::default_radius_for_point_announcements")]
    pub radius_for_point_announcements: f64,
    #[serde(default = "TouchDeviceAudioVectorOptions::default_line_distance")]
    pub distance_for_line_announcements: f64,
}

impl TouchDeviceAudioVectorOptions {
    const fn default_radius_for_point_announcements() -> f64 {
        5.0
    }

    const fn default_line_distance() -> f64 {
        5.0
    }
}

impl Default for TouchDeviceAudioVectorOptions {
    fn default() -> Self {
        Self {
            prefered_label_field: None,
            announce_leaving: true,
            announce_geometry_type: false,
            radius_for_point_announcements: Self::default_radius_for_point_announcements(),
            distance_for_line_announcements: Self::default_line_distance(),
        }
    }
}

#[derive(Debug)]
pub struct StatefulVectorLayer<'a> {
    pub layer: WrappedLayer<'a>,
    pub info: &'a mut StatefulVectorInfo,
}

impl<'a> StatefulVectorLayer<'a> {
    pub fn reproject(&self, output_name: &str, srs: Srs) -> std::io::Result<Output> {
        let srs = srs.try_to_gdal().unwrap();
        let mut command = Command::new("ogr2ogr");
        command.arg("-t_srs").arg(&srs.to_wkt().unwrap());
        command.arg(output_name).arg(&self.info.shared.name);
        command.output()
    }

    pub fn get_touch_device_layer_name(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.info.shared.name.hash(&mut hasher);
        self.layer.index.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn get_touch_device_info(&self) -> VectorInfo {
        VectorInfo {
            name: self.get_touch_device_layer_name(),
            settings: self.info.touch_device_settings.clone(),
        }
    }
}

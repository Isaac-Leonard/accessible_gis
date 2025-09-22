use std::{
    hash::{DefaultHasher, Hash, Hasher},
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};

use crate::{
    commands::SortOption,
    gdal_if::{Srs, WrappedLayer},
    web_socket::VectorInfo,
};

use super::shared::SharedInfo;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatefulVectorInfo {
    pub shared: SharedInfo,
    pub display: bool,
    #[serde(default)]
    pub desktop_settings: DesktopVectorOptions,
    #[serde(default)]
    pub touch_device_settings: TouchDeviceVectorOptions,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesktopVectorOptions {
    /// The index of each user selected feature for each layer of the dataset
    pub selected_feature: Option<usize>,
    /// The name of the field used to identify features
    pub primary_field_name: Option<String>,
    pub sort_features_by: SortOption,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TouchDeviceVectorOptions {
    pub audio: TouchDeviceAudioVectorOptions,
    pub visual: TouchDeviceVisualVectorOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TouchDeviceVisualVectorOptions {
    pub prefered_label_field: Option<String>,
    pub use_labels: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TouchDeviceAudioVectorOptions {
    pub prefered_label_field: Option<String>,
    pub announce_leaving: bool,
    pub announce_geometry_type: bool,
}

impl Default for TouchDeviceVectorOptions {
    fn default() -> Self {
        Self {
            audio: TouchDeviceAudioVectorOptions {
                prefered_label_field: None,
                announce_leaving: true,
                announce_geometry_type: false,
            },
            visual: TouchDeviceVisualVectorOptions {
                prefered_label_field: None,
                use_labels: false,
            },
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

    pub fn get_touch_device_info(&self) -> VectorInfo {
        let mut hasher = DefaultHasher::new();
        self.info.shared.name.hash(&mut hasher);
        self.layer.index.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());
        VectorInfo {
            name: hash,
            settings: self.info.touch_device_settings.clone(),
        }
    }
}

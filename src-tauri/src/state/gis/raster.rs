use std::process::{Command, Output};

use gdal::GeoTransformEx;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    gdal_if::{Srs, WrappedRasterBand},
    state::settings::AudioSettings,
};

use super::shared::SharedInfo;

#[derive(Debug, Clone, Default)]
pub enum ImageType {
    Dem,
    Red,
    Green,
    Blue,
    FarRed,
    #[default]
    Unknown,
}

pub struct StatefulRasterInfo {
    pub audio_settings: AudioSettings,
    pub shared: SharedInfo,
    pub image_type: ImageType,
    pub render: RenderMethod,
    pub ocr: bool,
}

#[derive(Clone, Copy, Debug, EnumIter, specta::Type, Serialize, Deserialize, PartialEq)]
pub enum RenderMethod {
    /// Try to use native browser image rendering or fall back to ImageJS
    Image,
    /// Render pure raster values mapped to 256 grey scale
    GDAL,
}

impl RenderMethod {
    pub fn get_variants() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}

pub struct StatefulRasterBand<'a> {
    pub band: WrappedRasterBand<'a>,
    pub info: &'a mut StatefulRasterInfo,
}

impl<'a> StatefulRasterBand<'a> {
    pub fn get_info_for_display(&mut self) -> RasterMetadata {
        let (width, height) = self.band.band().size();
        let geo_transform = self.band.geo_transform.unwrap();
        RasterMetadata {
            origin: (geo_transform[0], geo_transform[3]),
            width,
            height,
            // TODO: Should probably replace with x and y resolutions or even better just the full geotransform
            resolution: geo_transform[1],
        }
    }

    pub fn reproject(&self, output_name: &str, srs: Srs) -> std::io::Result<Output> {
        let srs = srs.try_to_gdal().unwrap();
        let mut command = Command::new("gdalwarp");
        command.arg("-t_srs").arg(&srs.to_wkt().unwrap());
        command.arg(&self.info.shared.name).arg(output_name);
        command.output()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct RasterMetadata {
    resolution: f64,
    width: usize,
    height: usize,
    origin: (f64, f64),
}

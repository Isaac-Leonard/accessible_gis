use std::{
    ffi::OsStr,
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    gdal_if::{Srs, WrappedRasterBand},
    state::settings::AudioSettings,
    web_socket::RasterDisplayInfo,
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

#[derive(Debug)]
pub struct StatefulRasterInfo {
    pub audio_settings: AudioSettings,
    pub shared: SharedInfo,
    pub image_type: ImageType,
    pub render: RenderMethod,
    pub ocr: bool,
}

#[derive(Clone, Copy, Debug, EnumIter, specta::Type, Serialize, Deserialize, PartialEq)]
pub enum RenderMethod {
    /// Displays the raw values of each cell on the screen
    RawData,
    /// Displays the raster as an image using Image-js and uses the raw values for sonification
    Combined,
    /// Displays the raster on the screen as an image using Image-js and uses the grey scale pixels of the image for sonification
    Image,
}

#[derive(Debug)]
pub struct StatefulRasterBand<'a> {
    pub band: WrappedRasterBand<'a>,
    pub info: &'a mut StatefulRasterInfo,
}

impl<'a> StatefulRasterBand<'a> {
    pub fn get_info_for_display(&self) -> RasterDisplayInfo {
        let (width, height) = self.band.band().size();
        let geo_transform = self.band.geo_transform.unwrap();
        let metadata = RasterMetadata {
            origin: (geo_transform[0], geo_transform[3]),
            width,
            height,
            // TODO: Should probably replace with x and y resolutions or even better just the full geotransform
            resolution: geo_transform[1],
            no_data_value: self.band.no_data_value(),
        };
        RasterDisplayInfo {
            kind: self.info.render,
            metadata,
        }
    }

    pub fn reproject<S: AsRef<OsStr>>(&self, output_name: S, srs: Srs) -> std::io::Result<Output> {
        let srs = srs.try_to_gdal().unwrap();
        let mut command = Command::new("gdalwarp");
        command.arg("-t_srs").arg(srs.to_wkt().unwrap());
        command.arg(&self.info.shared.name).arg(output_name);
        eprintln!("{:?}", command);
        command.output()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RasterMetadata {
    pub resolution: f64,
    pub width: usize,
    pub height: usize,
    pub origin: (f64, f64),
    pub no_data_value: Option<f64>,
}

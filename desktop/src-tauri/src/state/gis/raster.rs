use std::{
    ffi::OsStr,
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use tauri::AppHandle;

use crate::{
    files::get_random_temp_path,
    gdal_if::{Srs, WrappedDataset, WrappedRasterBand},
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
    pub wgs84_reprojected_file: Option<WrappedDataset>,
    pub audio_table: Option<AudioTable>,
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
    index: usize,
}

impl<'a> StatefulRasterBand<'a> {
    pub fn new(
        band: WrappedRasterBand<'a>,
        info: &'a mut StatefulRasterInfo,
        index: usize,
    ) -> Self {
        Self { band, info, index }
    }

    /// Returns the base 1 index of this raster band
    pub fn get_index(&self) -> usize {
        self.index
    }

    /// Gets the necessary information to display the raster and performs any required reprojections.
    /// As this is called before any actual data is sent to the touch device the reprojection is centralised here so other functions can just unwrap the wgs84_reprojected_dataset field on band.info.
    /// TODO: This isn't perfect, maybe we could replace with a LazyCell or something but this will do for now as it minimises reprojections and makes the code less fragile then it was before.
    pub fn get_info_for_display(&mut self, app: &AppHandle) -> RasterDisplayInfo {
        let index = self.get_index();
        let dataset = if let Some(dataset) = self.info.wgs84_reprojected_file.as_mut() {
            dataset
        } else {
            let reprojected_dataset_name = get_random_temp_path(app, "tif");
            std::fs::remove_file(&reprojected_dataset_name);
            dbg!(self.reproject(&reprojected_dataset_name, Srs::Epsg(4326)));
            let wgs84_dataset = WrappedDataset::open(reprojected_dataset_name).unwrap();
            self.info.wgs84_reprojected_file = Some(wgs84_dataset);
            self.info.wgs84_reprojected_file.as_mut().unwrap()
        };
        let band = dataset.get_raster(index).unwrap();
        eprintln!(
            "Reprojected display band to bounds: {:?}",
            band.get_bounds()
        );
        let (width, height) = band.band().size();
        let geo_transform = band.geo_transform.unwrap();
        let metadata = RasterMetadata {
            origin: (geo_transform[0], geo_transform[3]),
            width,
            height,
            // TODO: Should probably replace with x and y resolutions or even better just the full geotransform
            resolution: geo_transform[1],
            no_data_value: band.no_data_value(),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub enum AudioType {
    Frequency(f64),
    Silence,
    Speak(String),
    LinearMap,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    pixel_value: i64,
    audio: AudioType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioTable {
    mapping: Vec<Classification>,
    other: Classification,
}

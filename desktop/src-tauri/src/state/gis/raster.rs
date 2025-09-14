use std::{
    ffi::OsStr,
    process::{Command, Output},
};

use csv::ReaderBuilder;
use itertools::Itertools;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, Unexpected},
};
use strum::{EnumDiscriminants, EnumIter};
use tauri::{AppHandle, Manager};

use crate::{
    errors::ErrorDetails,
    files::get_random_temp_path,
    gdal_if::{Srs, WrappedDataset, WrappedRasterBand},
    state::settings::AudioSettings,
    web_socket::RasterDisplayInfo,
};

use super::shared::SharedInfo;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ImageType {
    Dem,
    Red,
    Green,
    Blue,
    FarRed,
    #[default]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatefulRasterInfo {
    pub audio_settings: AudioSettings,
    pub shared: SharedInfo,
    pub image_type: ImageType,
    pub render: RenderMethod,
    pub ocr: bool,
    #[serde(skip)]
    pub wgs84_reprojected_file: Option<WrappedDataset>,
    pub audio_table: Option<AudioTable>,
}

impl Clone for StatefulRasterInfo {
    /// Copies all fields but for the wgs84_reprojected_dataset as it cannot be stored
    fn clone(&self) -> Self {
        Self {
            audio_settings: self.audio_settings.clone(),
            shared: self.shared.clone(),
            image_type: self.image_type.clone(),
            render: self.render.clone(),
            ocr: self.ocr.clone(),
            wgs84_reprojected_file: None,
            audio_table: self.audio_table.clone(),
        }
    }
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
    pub fn get_info_for_display(
        &mut self,
        app: &AppHandle,
    ) -> Result<RasterDisplayInfo, ErrorDetails> {
        let index = self.get_index();
        let dataset = if let Some(dataset) = self.info.wgs84_reprojected_file.as_mut() {
            dataset
        } else {
            let reprojected_dataset_name = get_random_temp_path(app, "tif");
            // Deliberately ignore the result as it is almost certain to get an error as most paths should be unique.
            let _ = std::fs::remove_file(&reprojected_dataset_name);
            self.reproject(&reprojected_dataset_name, Srs::Epsg(4326))
                .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
            let wgs84_dataset = WrappedDataset::open(reprojected_dataset_name)
                .map_err(|err| ErrorDetails::OpenDatasetError(err))?;
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
            audio_table: self.info.audio_table.clone(),
        };
        Ok(RasterDisplayInfo {
            kind: self.info.render,
            metadata,
        })
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
    audio_table: Option<AudioTable>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type, EnumDiscriminants)]
#[serde(tag = "type", content = "value")]
#[strum_discriminants(derive(EnumIter, Serialize, Deserialize, specta::Type))]
pub enum AudioType {
    Frequency(f64),
    Silence,
    Speak(String),
    LinearMap,
    EscSound(usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioTable {
    entries: Vec<AudioType>,
    other: AudioType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct EscSound {
    pub filename: String,
    pub fold: String,
    pub target: String,
    pub category: String,
    #[serde(deserialize_with = "bool_from_str")]
    pub esc10: bool,
    pub src_file: String,
    pub take: char,
}

const ESC_META_PATH: &str = "esc-50/meta/esc50.csv";

pub fn load_esc_sounds(app: &AppHandle) -> Result<Vec<EscSound>, csv::Error> {
    let path = app
        .path()
        .resolve(ESC_META_PATH, tauri::path::BaseDirectory::Resource)
        .unwrap();
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
    reader.deserialize::<EscSound>().try_collect()
}

/// Converts a string to a boolean based on truthy and falsy values
/// Copied from This github comment: https://github.com/BurntSushi/rust-csv/issues/135#issuecomment-752783194
fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.to_lowercase().as_str() {
        "t" | "true" | "1" | "on" | "y" | "yes" => Ok(true),
        "f" | "false" | "0" | "off" | "n" | "no" => Ok(false),
        other => Err(de::Error::invalid_value(
            Unexpected::Str(other),
            &"Must be truthy (t, true, 1, on, y, yes) or falsey (f, false, 0, off, n, no)",
        )),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Foo {
    /// Some field
    bar: bool,
}

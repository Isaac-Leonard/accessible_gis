use crate::{gdal_if::WrappedRasterBand, state::settings::AudioSettings};

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
}

pub struct StatefulRasterBand<'a> {
    pub band: WrappedRasterBand<'a>,
    pub info: &'a mut StatefulRasterInfo,
}

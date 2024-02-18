use gdal::raster::{GdalDataType, RasterBand};
use ndarray::Array2;
use std::sync::mpsc::SyncSender;

use crate::audio::{get_audio, AudioMessage};

#[derive(Clone, PartialEq, Debug, Copy)]
pub struct RasterIndex {
    pub dataset: usize,
    pub raster: usize,
}

impl RasterIndex {
    pub fn new(dataset: usize, raster: usize) -> Self {
        Self { dataset, raster }
    }
}

pub fn read_raster_data(band: &RasterBand) -> Array2<f64> {
    match band.band_type() {
        GdalDataType::UInt8 => band
            .read_as_array::<u8>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),
        GdalDataType::UInt16 => band
            .read_as_array::<u16>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),
        GdalDataType::UInt32 => band
            .read_as_array::<u32>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Int8 => band
            .read_as_array::<i8>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Int16 => band
            .read_as_array::<i16>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Int32 => band
            .read_as_array::<i32>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Float32 => band
            .read_as_array::<f32>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Float64 => band
            .read_as_array::<f64>((0, 0), band.size(), band.size(), None)
            .unwrap(),

        _ => panic!("Unknown datatype in raster band"),
    }
}
#[derive(Clone)]
pub struct RawRasterData {
    pub data_type: String,
    pub data: Array2<f64>,
    pub min: f64,
    pub max: f64,
    pub no_data_value: Option<f64>,
    pub sender: SyncSender<AudioMessage>,
    pub index: RasterIndex,
}
impl RawRasterData {
    pub fn new(band: RasterBand, index: RasterIndex) -> Self {
        let band_type = band.band_type();
        let data = read_raster_data(&band);
        let min_max = band.compute_raster_min_max(false).unwrap();
        RawRasterData {
            data_type: band_type.name(),
            data,
            min: min_max.min,
            max: min_max.max,
            no_data_value: band.no_data_value(),
            sender: get_audio(),
            index,
        }
    }
}

impl PartialEq for RawRasterData {
    fn eq(&self, other: &Self) -> bool {
        self.data_type == other.data_type
            && self.no_data_value == other.no_data_value
            && self.data == other.data
            && self.min == other.min
            && self.max == other.max
    }
}

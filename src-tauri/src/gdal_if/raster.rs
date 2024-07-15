use gdal::{
    raster::{GdalDataType, RasterBand, ResampleAlg},
    GeoTransform, GeoTransformEx,
};
use geo_types::Point;
use itertools::Itertools;
use ndarray::Array2;
use proj::{Coord, Transform};
use serde::{Deserialize, Serialize};

pub struct WrappedRasterBand<'a> {
    pub band: RasterBand<'a>,
    pub geo_transform: Option<GeoTransform>,
    pub srs: Option<String>,
}

impl<'a> WrappedRasterBand<'a> {
    pub fn point_to_wgs84(&self, point: Point) -> Option<Point> {
        let point = self.geo_transform?.apply(point.x(), point.y());
        let point = Point::from_xy(point.0, point.1);
        point
            .transformed_crs_to_crs(self.srs.as_ref()?, "WGS84")
            .ok()
    }

    pub fn no_data_value(&self) -> Option<f64> {
        self.band.no_data_value()
    }

    pub fn band(&self) -> &RasterBand<'a> {
        &self.band
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum RasterData {
    UInt8(Vec<u8>),
    Int8(Vec<i8>),
    UInt16(Vec<u16>),
    Int16(Vec<i16>),
    UInt32(Vec<u32>),
    Int32(Vec<i32>),
    UInt64(Vec<u64>),
    Int64(Vec<i64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
}

impl RasterData {
    pub fn to_f64(self) -> Vec<f64> {
        match self {
            RasterData::UInt8(data) => data.into_iter().map_into().collect_vec(),
            RasterData::Int8(data) => data.into_iter().map_into().collect_vec(),
            RasterData::UInt16(data) => data.into_iter().map_into().collect_vec(),
            RasterData::Int16(data) => data.into_iter().map_into().collect_vec(),
            RasterData::UInt32(data) => data.into_iter().map_into().collect_vec(),
            RasterData::Int32(data) => data.into_iter().map_into().collect_vec(),
            // These next to are not fully convertible, they may have values higher then the largest precise floating point integer representation
            // We assume that is not the case for this function but Into is not implemented for the conversion
            RasterData::UInt64(data) => data.into_iter().map(|v| v as f64).collect_vec(),
            RasterData::Int64(data) => data.into_iter().map(|v| v as f64).collect_vec(),
            RasterData::Float32(data) => data.into_iter().map_into().collect_vec(),
            RasterData::Float64(data) => data,
        }
    }
}

pub fn read_raster_data(band: &RasterBand) -> Array2<f64> {
    match band.band_type() {
        GdalDataType::UInt8 => band
            .read_as::<u8>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),
        GdalDataType::UInt16 => band
            .read_as::<u16>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),
        GdalDataType::UInt32 => band
            .read_as::<u32>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Int8 => band
            .read_as::<i8>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Int16 => band
            .read_as::<i16>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Int32 => band
            .read_as::<i32>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Float32 => band
            .read_as::<f32>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap()
            .mapv_into_any(|x| x as f64),

        GdalDataType::Float64 => band
            .read_as::<f64>((0, 0), band.size(), band.size(), None)
            .unwrap()
            .to_array()
            .unwrap(),
        _ => panic!("Unknown datatype in raster band"),
    }
}

pub fn read_raster_data_enum(band: &RasterBand) -> Option<RasterData> {
    let data_type = band.band_type();
    match data_type {
        GdalDataType::UInt8 => {
            let ((_width, _height), data) = band
                .read_as::<u8>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt8(data))
        }
        GdalDataType::Int8 => {
            let ((_width, _height), data) = band
                .read_as::<i8>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int8(data))
        }
        GdalDataType::UInt16 => {
            let ((_width, _height), data) = band
                .read_as::<u16>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt16(data))
        }
        GdalDataType::Int16 => {
            let ((_width, _height), data) = band
                .read_as::<i16>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int16(data))
        }
        GdalDataType::UInt32 => {
            let ((_width, _height), data) = band
                .read_as::<u32>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt32(data))
        }
        GdalDataType::Int32 => {
            let ((_width, _height), data) = band
                .read_as::<i32>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int32(data))
        }
        GdalDataType::UInt64 => {
            let ((_width, _height), data) = band
                .read_as::<u64>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt64(data))
        }
        GdalDataType::Int64 => {
            let ((_width, _height), data) = band
                .read_as::<i64>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int64(data))
        }
        GdalDataType::Float32 => {
            let ((_width, _height), data) = band
                .read_as::<f32>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Float32(data))
        }
        GdalDataType::Float64 => {
            let ((_width, _height), data) = band
                .read_as::<f64>((0, 0), band.size(), band.size(), None)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Float64(data))
        }
        _ => None,
    }
}

pub fn read_raster_data_enum_as(
    band: &RasterBand,
    window: (isize, isize),
    size: (usize, usize),
    buffer_size: (usize, usize),
    e_resample_alg: Option<ResampleAlg>,
) -> Option<RasterData> {
    let data_type = band.band_type();
    match data_type {
        GdalDataType::UInt8 => {
            let ((_width, _height), data) = band
                .read_as::<u8>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt8(data))
        }
        GdalDataType::Int8 => {
            let ((_width, _height), data) = band
                .read_as::<i8>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int8(data))
        }
        GdalDataType::UInt16 => {
            let ((_width, _height), data) = band
                .read_as::<u16>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt16(data))
        }
        GdalDataType::Int16 => {
            let ((_width, _height), data) = band
                .read_as::<i16>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int16(data))
        }
        GdalDataType::UInt32 => {
            let ((_width, _height), data) = band
                .read_as::<u32>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt32(data))
        }
        GdalDataType::Int32 => {
            let ((_width, _height), data) = band
                .read_as::<i32>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int32(data))
        }
        GdalDataType::UInt64 => {
            let ((_width, _height), data) = band
                .read_as::<u64>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::UInt64(data))
        }
        GdalDataType::Int64 => {
            let ((_width, _height), data) = band
                .read_as::<i64>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Int64(data))
        }
        GdalDataType::Float32 => {
            let ((_width, _height), data) = band
                .read_as::<f32>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Float32(data))
        }
        GdalDataType::Float64 => {
            let ((_width, _height), data) = band
                .read_as::<f64>(window, size, buffer_size, e_resample_alg)
                .ok()?
                .into_shape_and_vec();
            Some(RasterData::Float64(data))
        }
        _ => None,
    }
}

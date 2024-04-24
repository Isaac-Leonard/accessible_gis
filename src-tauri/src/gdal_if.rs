use gdal::{
    raster::{GdalDataType, RasterBand},
    vector::{Envelope as GdalEnvelope, Feature, FieldValue as GdalFieldValue},
    Dataset, Driver, DriverManager, Metadata,
};
use geo::{Closest, ClosestPoint, GeodesicDistance};
use geo_types::{Geometry as GeoGeometry, Point};
use itertools::Itertools;
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn get_driver_for_file<P: AsRef<Path>>(path: P) -> Option<Driver> {
    (0..DriverManager::count())
        .map(|index| DriverManager::get_driver(index).unwrap())
        .find(|driver| {
            let meta = driver
                .metadata()
                .into_iter()
                .find(|meta| meta.key == "gdal.DMD_EXTENSIONS");
            let Some(meta) = meta else { return false };
            let mut extentions = meta.value.split(' ');
            extentions.any(|x| x == path.as_ref().extension().unwrap().to_str().unwrap())
        })
}

#[derive(Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Field {
    pub name: String,
    #[serde(flatten)]
    pub value: FieldValue,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, specta::Type)]
#[serde(tag = "type", content = "value")]
pub enum FieldValue {
    Integer(i32),
    IntegerList(Vec<i32>),
    Integer64(i64),
    Integer64List(Vec<i64>),
    String(String),
    StringList(Vec<String>),
    Real(f64),
    RealList(Vec<f64>),
    // TODO: Handle dates safely
    Date(String),
    DateTime(String),
    None,
}

impl ToString for FieldValue {
    fn to_string(&self) -> String {
        match self {
            FieldValue::Integer(val) => val.to_string(),
            FieldValue::IntegerList(val) => val.iter().map(ToString::to_string).join(", "),
            FieldValue::Integer64(val) => val.to_string(),
            FieldValue::Integer64List(val) => val.iter().map(ToString::to_string).join(", "),
            FieldValue::String(val) => val.to_owned(),
            FieldValue::StringList(val) => val.join(", "),
            FieldValue::Real(val) => val.to_string(),
            FieldValue::RealList(val) => val.iter().map(ToString::to_string).join(", "),
            FieldValue::Date(val) => val.to_owned(),
            FieldValue::DateTime(val) => val.to_owned(),
            FieldValue::None => "Empty".to_owned(),
        }
    }
}

impl From<GdalFieldValue> for FieldValue {
    fn from(value: GdalFieldValue) -> Self {
        match value {
            GdalFieldValue::IntegerValue(val) => Self::Integer(val),
            GdalFieldValue::IntegerListValue(val) => Self::IntegerList(val),
            GdalFieldValue::Integer64Value(val) => Self::Integer64(val),
            GdalFieldValue::Integer64ListValue(val) => Self::Integer64List(val),
            GdalFieldValue::StringValue(val) => Self::String(val),
            GdalFieldValue::StringListValue(val) => Self::StringList(val),
            GdalFieldValue::RealValue(val) => Self::Real(val),
            GdalFieldValue::RealListValue(val) => Self::RealList(val),
            GdalFieldValue::DateValue(val) => Self::Date(val.to_string()),
            GdalFieldValue::DateTimeValue(val) => Self::DateTime(val.to_string()),
        }
    }
}

impl From<Option<GdalFieldValue>> for FieldValue {
    fn from(value: Option<GdalFieldValue>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::None,
        }
    }
}

impl From<(String, Option<GdalFieldValue>)> for FieldValue {
    fn from(value: (String, Option<GdalFieldValue>)) -> Self {
        Self::from(value.1)
    }
}

impl From<(String, Option<GdalFieldValue>)> for Field {
    fn from(value: (String, Option<GdalFieldValue>)) -> Self {
        Self {
            name: value.0,
            value: FieldValue::from(value.1),
        }
    }
}

pub struct WrappedDataset {
    pub file_name: String,
    pub dataset: Dataset,
    pub raster_data: Option<Array2<f64>>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFeatureInfo {
    pub fields: Vec<Field>,
    pub geometry: GeoGeometry,
}

impl LocalFeatureInfo {
    pub fn get_field(&self, name: &str) -> Option<String> {
        Some(
            self.fields
                .iter()
                .find(|f| f.name == name)?
                .value
                .to_string()
                .clone(),
        )
    }

    /// Note, this operates on WGS84 coordinates
    pub fn nearest_point(&self, point: &Point) -> Option<f64> {
        Some(match self.geometry.closest_point(point) {
            Closest::SinglePoint(p) => p.geodesic_distance(point),
            Closest::Intersection(_) => 0.0,
            Closest::Indeterminate => return None,
        })
    }

    pub fn get_name(&self) -> Option<String> {
        Some(
            self.fields
                .iter()
                .find(|f| f.name == "name")?
                .value
                .to_string()
                .clone(),
        )
    }
}

impl From<Feature<'_>> for LocalFeatureInfo {
    fn from(feature: Feature) -> Self {
        LocalFeatureInfo {
            geometry: feature.geometry().unwrap().to_geo().unwrap(),
            fields: feature.fields().map(Into::into).collect(),
        }
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, specta::Type)]
pub struct Envelope {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl From<GdalEnvelope> for Envelope {
    fn from(value: GdalEnvelope) -> Self {
        Self {
            min_x: value.MinX,
            max_x: value.MaxX,
            min_y: value.MinY,
            max_y: value.MaxY,
        }
    }
}

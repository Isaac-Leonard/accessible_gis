use gdal::vector::{Feature, FieldValue as GdalFieldValue};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::FeatureInfo;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, specta::Type)]
pub struct Field {
    pub name: String,
    #[serde(flatten)]
    pub value: FieldValue,
}

impl<'a> From<Feature<'a>> for FeatureInfo {
    fn from(value: Feature<'a>) -> Self {
        Self {
            fields: get_fields(&value),
            geometry: value.geometry().unwrap().to_geo().unwrap().into(),
        }
    }
}

pub fn get_fields(feature: &Feature) -> Vec<Field> {
    feature
        .fields()
        .map(|(name, value)| Field {
            name,
            value: value.into(),
        })
        .collect::<Vec<_>>()
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, specta::Type)]
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

impl From<FieldValue> for GdalFieldValue {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Integer(val) => Self::IntegerValue(val),
            FieldValue::IntegerList(val) => Self::IntegerListValue(val),
            FieldValue::Integer64(val) => Self::Integer64Value(val),
            FieldValue::Integer64List(val) => Self::Integer64ListValue(val),
            FieldValue::String(val) => Self::StringValue(val),
            FieldValue::StringList(val) => Self::StringListValue(val),
            FieldValue::Real(val) => Self::RealValue(val),
            FieldValue::RealList(val) => Self::RealListValue(val),
            FieldValue::Date(val) => Self::DateValue(val.parse().unwrap()),
            FieldValue::DateTime(val) => Self::DateTimeValue(val.parse().unwrap()),
            FieldValue::None => unreachable!(),
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

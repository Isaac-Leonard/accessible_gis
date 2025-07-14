//! Copy of GdalError with extra derives
//! GDAL Error Types

use gdal::errors::GdalError;
use gdal::raster::ExtendedDataTypeClass;
use gdal_sys::{CPLErr, GDALExtendedDataTypeClass, OGRErr, OGRFieldType, OGRwkbGeometryType};
use ndarray::ShapeError;
use serde::{Deserialize, Serialize};
use std::ffi::{CString, IntoStringError, c_int};

use crate::errors::MyUtf8Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", content = "data")]
pub enum MyGdalError {
    FfiNulError(MyNulError),
    FfiIntoStringError(MyIntoStringError),
    StrUtf8Error(MyUtf8Error),
    NdarrayShapeError(MyShapeError),
    CplError {
        class: CPLErr::Type,
        number: c_int,
        msg: String,
    },
    NullPointer {
        method_name: &'static str,
        msg: String,
    },
    CastToF64Error,
    OgrError {
        err: OGRErr::Type,
        method_name: &'static str,
    },
    UnhandledFieldType {
        field_type: OGRFieldType::Type,
        method_name: &'static str,
    },
    InvalidFieldName {
        field_name: String,
        method_name: &'static str,
    },
    InvalidFieldIndex {
        index: usize,
        method_name: &'static str,
    },
    UnlinkedGeometry {
        method_name: &'static str,
    },
    InvalidCoordinateRange {
        from: String,
        to: String,
        msg: Option<String>,
    },
    AxisNotFoundError {
        key: String,
        method_name: &'static str,
    },
    UnsupportedGdalGeometryType(OGRwkbGeometryType::Type),
    UnlinkMemFile {
        file_name: String,
    },
    BadArgument(String),
    DateError(String),
    UnsupportedMdDataType {
        data_type: MyExtendedDataTypeClass,
        method_name: &'static str,
    },
    IntConversionError(()),
    BufferSizeMismatch(usize, (usize, usize)),
}

impl From<GdalError> for MyGdalError {
    fn from(err: GdalError) -> Self {
        match err {
            GdalError::FfiNulError(e) => MyGdalError::FfiNulError(e.into()),
            GdalError::FfiIntoStringError(e) => MyGdalError::FfiIntoStringError(e.into()),
            GdalError::StrUtf8Error(e) => MyGdalError::StrUtf8Error(e.into()),
            GdalError::NdarrayShapeError(e) => MyGdalError::NdarrayShapeError(e.into()),
            GdalError::CplError { class, number, msg } => {
                MyGdalError::CplError { class, number, msg }
            }
            GdalError::NullPointer { method_name, msg } => {
                MyGdalError::NullPointer { method_name, msg }
            }
            GdalError::CastToF64Error => MyGdalError::CastToF64Error,
            GdalError::OgrError { err, method_name } => MyGdalError::OgrError { err, method_name },
            GdalError::UnhandledFieldType {
                field_type,
                method_name,
            } => MyGdalError::UnhandledFieldType {
                field_type,
                method_name,
            },
            GdalError::InvalidFieldName {
                field_name,
                method_name,
            } => MyGdalError::InvalidFieldName {
                field_name,
                method_name,
            },
            GdalError::InvalidFieldIndex { index, method_name } => {
                MyGdalError::InvalidFieldIndex { index, method_name }
            }
            GdalError::UnlinkedGeometry { method_name } => {
                MyGdalError::UnlinkedGeometry { method_name }
            }
            GdalError::InvalidCoordinateRange { from, to, msg } => {
                MyGdalError::InvalidCoordinateRange { from, to, msg }
            }
            GdalError::AxisNotFoundError { key, method_name } => {
                MyGdalError::AxisNotFoundError { key, method_name }
            }
            GdalError::UnsupportedGdalGeometryType(t) => {
                MyGdalError::UnsupportedGdalGeometryType(t)
            }
            GdalError::UnlinkMemFile { file_name } => MyGdalError::UnlinkMemFile { file_name },
            GdalError::BadArgument(msg) => MyGdalError::BadArgument(msg),
            GdalError::DateError(msg) => MyGdalError::DateError(msg),
            GdalError::UnsupportedMdDataType {
                data_type,
                method_name,
            } => MyGdalError::UnsupportedMdDataType {
                data_type: data_type.into(),
                method_name,
            },
            GdalError::IntConversionError(_e) => MyGdalError::IntConversionError(()),
            GdalError::BufferSizeMismatch(a, b) => MyGdalError::BufferSizeMismatch(a, b),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct MyNulError(usize, Vec<u8>);
impl From<std::ffi::NulError> for MyNulError {
    fn from(value: std::ffi::NulError) -> Self {
        Self(value.nul_position(), value.into_vec())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum MyExtendedDataTypeClass {
    Compound = GDALExtendedDataTypeClass::GEDTC_COMPOUND as isize,
    Numeric = GDALExtendedDataTypeClass::GEDTC_NUMERIC as isize,
    String = GDALExtendedDataTypeClass::GEDTC_STRING as isize,
}

impl From<ExtendedDataTypeClass> for MyExtendedDataTypeClass {
    fn from(value: ExtendedDataTypeClass) -> Self {
        match value {
            ExtendedDataTypeClass::Compound => MyExtendedDataTypeClass::Compound,
            ExtendedDataTypeClass::Numeric => MyExtendedDataTypeClass::Numeric,
            ExtendedDataTypeClass::String => MyExtendedDataTypeClass::String,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, specta::Type)]
pub struct MyIntoStringError {
    pub inner: CString,
    pub error: MyUtf8Error,
}

impl From<IntoStringError> for MyIntoStringError {
    fn from(value: IntoStringError) -> Self {
        Self {
            error: value.utf8_error().into(),
            inner: value.into_cstring(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub enum MyShapeError {
    IncompatibleShape = 1,
    IncompatibleLayout,
    RangeLimited,
    OutOfBounds,
    Unsupported,
    Overflow,
    Other,
}

impl From<ShapeError> for MyShapeError {
    fn from(value: ShapeError) -> Self {
        use ndarray::ErrorKind;
        match value.kind() {
            ErrorKind::IncompatibleShape => Self::IncompatibleShape,
            ErrorKind::IncompatibleLayout => Self::IncompatibleLayout,
            ErrorKind::RangeLimited => Self::RangeLimited,
            ErrorKind::OutOfBounds => Self::OutOfBounds,
            ErrorKind::Unsupported => Self::Unsupported,
            ErrorKind::Overflow => Self::Overflow,
            _ => Self::Other,
        }
    }
}

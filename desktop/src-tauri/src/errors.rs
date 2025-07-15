use std::str::Utf8Error;

use serde::{Deserialize, Serialize};

use crate::{
    commands::EditDatasetError,
    gdal_if::{DatasetCreationError, OpenDatasetError},
    tools::DemClassificationError,
};

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct ApplicationError {
    #[serde(flatten)]
    pub details: ErrorDetails,
    pub read: bool,
    pub id: uuid::Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum ErrorDetails {
    ExternalProgramError(DemClassificationError),
    DatasetCreationError(DatasetCreationError),
    EditDatasetError(EditDatasetError),
    OpenDatasetError(OpenDatasetError),
    CsvError(MyCsvError),
    TouchDeviceError(String),
    IoError(String),
    SerdeError(String),
    Other(String),
}

impl From<ErrorDetails> for ApplicationError {
    fn from(value: ErrorDetails) -> Self {
        Self {
            details: value,
            read: false,
            id: uuid::Uuid::new_v4(),
        }
    }
}

impl ErrorDetails {
    pub fn into_application_error(self) -> ApplicationError {
        self.into()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct FailedToRun(String);

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct MyCsvError(Box<MyCsvErrorKind>);

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub enum MyCsvErrorKind {
    /// An I/O error that occurred while reading CSV data.
    Io(String),
    /// A UTF-8 decoding error that occured while reading CSV data into Rust
    /// `String`s.
    Utf8 {
        /// The position of the record in which this error occurred, if
        /// available.
        pos: Option<MyCsvPosition>,
        /// The corresponding UTF-8 error.
        err: MyUtf8Error,
    },
    /// This error occurs when two records with an unequal number of fields
    /// are found. This error only occurs when the `flexible` option in a
    /// CSV reader/writer is disabled.
    UnequalLengths {
        /// The position of the first record with an unequal number of fields
        /// to the previous record, if available.
        pos: Option<MyCsvPosition>,
        /// The expected number of fields in a record. This is the number of
        /// fields in the record read prior to the record indicated by
        /// `pos`.
        expected_len: u64,
        /// The number of fields in the bad record.
        len: u64,
    },
    /// This error occurs when either the `byte_headers` or `headers` methods
    /// are called on a CSV reader that was asked to `seek` before it parsed
    /// the first record.
    Seek,
    /// An error of this kind occurs only when using the Serde serializer.
    Serialize(String),
    /// An error of this kind occurs only when performing automatic
    /// deserialization with serde.
    Deserialize {
        /// The position of this error, if available.
        pos: Option<MyCsvPosition>,
        /// The deserialization error.
        err: MyCsvDeserializeError,
    },
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    __Nonexhaustive,
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct MyUtf8Error {
    pub valid_up_to: usize,
    pub error_len: Option<usize>,
}

impl From<Utf8Error> for MyUtf8Error {
    fn from(value: Utf8Error) -> Self {
        Self {
            valid_up_to: value.valid_up_to(),
            error_len: value.error_len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct MyCsvPosition {
    byte: u64,
    line: u64,
    record: u64,
}

impl From<csv::Position> for MyCsvPosition {
    fn from(value: csv::Position) -> Self {
        Self {
            line: value.line(),
            byte: value.byte(),
            record: value.record(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct MyCsvDeserializeError {
    field: Option<u64>,
    kind: MyCsvDeserializeErrorKind,
}

impl From<csv::DeserializeError> for MyCsvDeserializeError {
    fn from(value: csv::DeserializeError) -> Self {
        Self {
            field: value.field().clone(),
            kind: value.kind().clone().into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub enum MyCsvDeserializeErrorKind {
    Message(String),
    Unsupported(String),
    UnexpectedEndOfRow,
    InvalidUtf8(MyUtf8Error),
    ParseBool(String),
    ParseInt(String),
    ParseFloat(String),
}

impl From<csv::DeserializeErrorKind> for MyCsvDeserializeErrorKind {
    fn from(value: csv::DeserializeErrorKind) -> Self {
        use csv::DeserializeErrorKind;
        match value {
            DeserializeErrorKind::Message(message) => Self::Message(message),
            DeserializeErrorKind::Unsupported(message) => Self::Unsupported(message),
            DeserializeErrorKind::UnexpectedEndOfRow => Self::UnexpectedEndOfRow,
            DeserializeErrorKind::InvalidUtf8(utf8_error) => Self::InvalidUtf8(utf8_error.into()),
            DeserializeErrorKind::ParseBool(err) => Self::ParseBool(err.to_string()),
            DeserializeErrorKind::ParseInt(err) => Self::ParseInt(err.to_string()),
            DeserializeErrorKind::ParseFloat(err) => Self::ParseFloat(err.to_string()),
        }
    }
}

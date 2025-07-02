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

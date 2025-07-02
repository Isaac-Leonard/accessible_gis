use serde::{Deserialize, Serialize};

use crate::tools::DemClassificationError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ApplicationError {
    #[serde(flatten)]
    pub details: ErrorDetails,
    pub read: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum ErrorDetails {
    ExternalProgramError(DemClassificationError),
    Other(String),
}

impl From<ErrorDetails> for ApplicationError {
    fn from(value: ErrorDetails) -> Self {
        Self {
            details: value,
            read: false,
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

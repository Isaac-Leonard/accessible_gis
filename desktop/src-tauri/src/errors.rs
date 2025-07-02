use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum ApplicationError {
    ExternalProgramError,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct FailedToRun(String);

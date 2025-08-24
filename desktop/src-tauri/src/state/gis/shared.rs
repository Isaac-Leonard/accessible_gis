use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Currently just a place holder for future data
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SharedInfo {
    pub name: PathBuf,
}

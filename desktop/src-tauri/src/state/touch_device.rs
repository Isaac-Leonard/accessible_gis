use serde::{Deserialize, Serialize};

use super::colours::{CssColour, NamedColour};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct TouchDeviceSettings {
    pub background_colour: CssColour,
}

impl Default for TouchDeviceSettings {
    fn default() -> Self {
        Self {
            background_colour: CssColour::Named(NamedColour::Black),
        }
    }
}

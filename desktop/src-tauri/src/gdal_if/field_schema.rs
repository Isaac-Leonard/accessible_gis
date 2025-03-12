use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub enum FieldType {
    /// Simple 32 bit integer
    OFTInteger = 0,
    /// List of 32 bit integers
    OFTIntegerList = 1,
    /// Double precision floating point
    OFTReal = 2,
    /// List of doubles
    OFTRealList = 3,
    /// String of ascii chars
    OFTString = 4,
    /// Array of strings
    OFTStringList = 5,
    /// Deprecated
    OFTWideString = 6,
    /// Deprecated
    OFTWideStringList = 7,
    /// Raw binary data
    OFTBinary = 8,
    /// Date
    OFTDate = 9,
    /// Time
    OFTTime = 10,
    /// Date and time
    OFTDateTime = 11,
    /// Single 64 bit integer
    OFTInteger64 = 12,
    /// List of 64 bit integers
    OFTInteger64List = 13,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FieldSchema {
    pub name: String,
    pub field_type: Option<FieldType>,
}

impl TryFrom<u32> for FieldType {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use FieldType::*;
        Ok(match value {
            0 => OFTInteger,
            1 => OFTIntegerList,
            2 => OFTReal,
            3 => OFTRealList,
            4 => OFTString,
            5 => OFTStringList,
            6 => OFTWideString,
            7 => OFTWideStringList,
            8 => OFTBinary,
            9 => OFTDate,
            10 => OFTTime,
            11 => OFTDateTime,
            12 => OFTInteger64,
            13 => OFTInteger64List,
            _ => return Err(()),
        })
    }
}

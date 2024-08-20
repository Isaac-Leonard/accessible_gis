use crate::gdal_if::WrappedLayer;

use super::shared::SharedInfo;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct StatefulVectorInfo {
    /// The index of each user selected feature for each layer of the dataset
    pub selected_feature: Option<usize>,
    /// The name of the field used to identify features
    pub primary_field_name: Option<String>,
    pub shared: SharedInfo,
}

pub struct StatefulVectorLayer<'a> {
    pub layer: WrappedLayer<'a>,
    pub info: &'a mut StatefulVectorInfo,
}

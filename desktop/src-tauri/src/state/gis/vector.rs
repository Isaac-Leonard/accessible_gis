use std::process::{Command, Output};

use crate::gdal_if::{Srs, WrappedLayer};

use super::shared::SharedInfo;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct StatefulVectorInfo {
    /// The index of each user selected feature for each layer of the dataset
    pub selected_feature: Option<usize>,
    /// The name of the field used to identify features
    pub primary_field_name: Option<String>,
    pub shared: SharedInfo,
    pub display: bool,
}

pub struct StatefulVectorLayer<'a> {
    pub layer: WrappedLayer<'a>,
    pub info: &'a mut StatefulVectorInfo,
}

impl<'a> StatefulVectorLayer<'a> {
    pub fn reproject(&self, output_name: &str, srs: Srs) -> std::io::Result<Output> {
        let srs = srs.try_to_gdal().unwrap();
        let mut command = Command::new("ogr2ogr");
        command.arg("-t_srs").arg(&srs.to_wkt().unwrap());
        command.arg(output_name).arg(&self.info.shared.name);
        command.output()
    }
}

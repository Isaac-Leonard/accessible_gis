use std::{
    ffi::{CStr, OsStr},
    process::{Command, Output},
};

use gdal::vector::{Layer, LayerAccess};

use crate::errors::ErrorDetails;

use super::{dataset::Srs, field_schema::FieldSchema};

pub struct WrappedLayer<'a> {
    pub layer: Layer<'a>,
    pub index: usize,
}

impl<'a> LayerExt for WrappedLayer<'a> {
    fn get_field_names(&self) -> Vec<String> {
        <Layer<'a> as LayerExt>::get_field_names(&self.layer)
    }

    fn get_field_schema(&self) -> Vec<FieldSchema> {
        <Layer<'a> as LayerExt>::get_field_schema(&self.layer)
    }
}

impl<'a> WrappedLayer<'a> {
    pub fn layer(&mut self) -> &mut Layer<'a> {
        &mut self.layer
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

pub trait LayerExt {
    fn get_field_names(&self) -> Vec<String>;
    fn get_field_schema(&self) -> Vec<FieldSchema>;
}

impl<'a> LayerExt for Layer<'a> {
    fn get_field_names(&self) -> Vec<String> {
        self.defn().fields().map(|x| x.name()).collect()
    }

    fn get_field_schema(&self) -> Vec<FieldSchema> {
        self.defn()
            .fields()
            .map(|x| FieldSchema {
                name: x.name(),
                field_type: x.field_type().try_into().ok(),
            })
            .collect()
    }
}

fn get_layer_name(layer: &Layer) -> Option<String> {
    unsafe {
        let defn = layer.defn().c_defn();
        let ptr = gdal_sys::OGR_FD_GetName(defn);
        if ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ptr).to_string_lossy().to_string())
        }
    }
}

pub fn merge_layers(
    names: Vec<impl AsRef<OsStr>>,
    single: bool,
    srs: Srs,
    output_name: impl AsRef<OsStr>,
    overwrite: bool,
) -> Result<Output, ErrorDetails> {
    let target_srs = srs
        .try_to_gdal()
        .map_err(|err| ErrorDetails::Other(err.to_string()))?
        .to_wkt()
        .map_err(|err| ErrorDetails::Other(err.to_string()))?;
    let mut command = Command::new("ogrmerge");
    if single {
        command.arg("-single");
    }
    if overwrite {
        command.arg("-overwrite_ds");
    }
    command.arg("-t_srs").arg(target_srs);
    command.arg("-o").arg(output_name);
    command.args(names);
    command
        .output()
        .map_err(|err| ErrorDetails::IoError(err.to_string()))
}

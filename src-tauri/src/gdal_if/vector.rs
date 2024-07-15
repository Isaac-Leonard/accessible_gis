use std::ffi::CStr;

use gdal::vector::{Layer, LayerAccess};

use super::field_schema::FieldSchema;

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

fn get_layer_name(layer: &Layer) -> String {
    unsafe {
        let defn = layer.defn().c_defn();
        CStr::from_ptr(gdal_sys::OGR_FD_GetName(defn))
            .to_string_lossy()
            .to_string()
    }
}

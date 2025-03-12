use std::{ffi::CString, process::Command};

use gdal::vector::{LayerAccess, ToGdal};
use geo_types::Geometry as GeoGeometry;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    gdal_if::{FieldType, LayerIndex},
    state::AppState,
    web_socket::{AppMessage, TouchDevice},
    FeatureInfo,
};

#[tauri::command]
#[specta::specta]
pub fn copy_features(features: Vec<usize>, name: &str, state: AppState) {
    state.with_current_dataset_mut(|ds, _| {
        let input_name = ds.dataset.file_name.clone();

        let mut command = Command::new("ogr2ogr");
        command
            .arg("-where")
            .arg(format!("fid in ({})", features.into_iter().join(", ")));
        command.arg(name).arg(&input_name);
        let output = command.output().unwrap();
        eprint!("{:?}", output)
    });
}

#[tauri::command]
#[specta::specta]
pub fn simplify_layer(tolerance: f64, name: String, state: AppState) {
    let tolerance_str = CString::new(tolerance.to_string()).unwrap();
    eprintln!("{}", unsafe { gdal_sys::CPLAtof(tolerance_str.as_ptr()) });
    eprintln!("{}", tolerance);
    state.with_current_dataset_mut(|ds, _| {
        let input_name = ds.dataset.file_name.clone();
        let mut command = Command::new("ogr2ogr");
        command.arg("-simplify").arg(tolerance.to_string());
        command.arg(name).arg(&input_name);
        let output = command.output().unwrap();
        eprint!("{:?}", output)
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_name_field(field: String, state: AppState) {
    state
        .with_current_vector_layer(|layer| {
            layer.info.primary_field_name = Some(field);
        })
        .expect("Tried to edit nonexistant dataset");
}

#[tauri::command]
#[specta::specta]
pub fn add_feature_to_layer(feature: FeatureInfo, state: AppState) -> Result<(), String> {
    state
        .with_current_vector_layer(|layer| {
            let geometry = feature
                .geometry
                .ok_or("Cannot create feature with null geometry")?;
            let geom = GeoGeometry::from(geometry)
                .to_gdal()
                .map_err(|_| "Failed to convert geometry to gdal geometry")?;
            let fields = feature
                .fields
                .iter()
                .flat_map(|field| {
                    Some((
                        field.name.as_str(),
                        gdal::vector::FieldValue::from(field.value.clone()),
                    ))
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();

            layer
                .layer
                .layer()
                .create_feature_fields(geom, &fields.0, &fields.1)
                .inspect_err(|e| eprintln!("{:?}", e))
                .map_err(|_| "Failed to add fields to schema".to_string())?;
            eprintln!(
                "{:?}",
                FeatureInfo::from(layer.layer.layer().features().last().unwrap())
            );
            Ok(())
        })
        .ok_or_else(|| "Tried to add feature to layer when state is uninitialised".to_string())?
}

#[tauri::command]
#[specta::specta]
pub fn add_field_to_schema(
    name: String,
    field_type: FieldType,
    state: AppState,
) -> Result<(), String> {
    state
        .with_current_vector_layer(move |layer| {
            layer
                .layer
                .layer()
                .create_defn_fields(&[(&name, field_type as u32)])
                .inspect_err(|e| eprintln!("{:?}", e))
                .map_err(|_| "Failed to add fields to schema".to_string())
        })
        .ok_or_else(|| "Tried to add field to schema when state is uninitialised".to_string())?
}

#[tauri::command]
#[specta::specta]
pub fn set_feature_index(index: usize, state: AppState) -> Result<(), String> {
    state
        .with_current_vector_layer(|layer| layer.info.selected_feature = Some(index))
        .ok_or_else(|| "error".to_string())
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FeatureIndex {
    layer: LayerIndex,
    feature: usize,
}

#[tauri::command]
#[specta::specta]
pub fn set_layer_index(index: LayerIndex, state: AppState) {
    state
        .with_current_dataset_mut(|ds, _| {
            ds.layer_index = Some(index);
        })
        .expect("Tried to set layer index on nonexistant dataset");
}

#[tauri::command]
#[specta::specta]
pub fn set_display_vector(state: AppState) {
    state
        .with_current_vector_layer(|layer| layer.info.display = true)
        .expect("No vector found when trying to set display");
}

#[tauri::command]
#[specta::specta]
pub fn set_prefered_display_fields(
    fields: Vec<String>,
    state: AppState,
    device: State<TouchDevice>,
) {
    state.with_lock(|state| {
        state.prefered_display_fields = fields;
        device.send(AppMessage::Gis(state.get_touch_device_settings().unwrap()));
    });
}

use std::{ffi::CString, process::Command};

use gdal::vector::{Feature, FieldValue, LayerAccess, ToGdal};
use geo_types::Geometry as GeoGeometry;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    FeatureInfo,
    errors::ErrorDetails,
    gdal_if::{FieldType, LayerIndex},
    state::AppState,
    web_socket::{AppMessage, TouchDevice},
};

#[tauri::command]
#[specta::specta]
pub fn copy_features(features: Vec<usize>, name: &str, state: AppState) {
    state.with_current_dataset_mut_fallible(|ds, _| {
        let input_name = ds.dataset.file_name.clone();

        let mut command = Command::new("ogr2ogr");
        command
            .arg("-where")
            .arg(format!("fid in ({})", features.into_iter().join(", ")));
        command.arg(name).arg(&input_name);
        let output = command
            .output()
            .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        eprint!("{:?}", output);
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn simplify_layer(tolerance: f64, name: String, state: AppState) {
    let tolerance_str = CString::new(tolerance.to_string()).unwrap();
    eprintln!("{}", unsafe { gdal_sys::CPLAtof(tolerance_str.as_ptr()) });
    eprintln!("{}", tolerance);
    state.with_current_dataset_mut_fallible(|ds, _| {
        let input_name = ds.dataset.file_name.clone();
        let mut command = Command::new("ogr2ogr");
        command.arg("-simplify").arg(tolerance.to_string());
        command.arg(name).arg(&input_name);
        let output = command
            .output()
            .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        eprint!("{output:?}",);
        Ok(())
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
pub fn add_feature_to_layer(feature: FeatureInfo, state: AppState) {
    eprintln!("{feature:?}");
    state.with_current_vector_layer_fallible(|layer| {
        let geom = feature
            .geometry
            .map(|geom| {
                GeoGeometry::from(geom).to_gdal().map_err(|err| {
                    ErrorDetails::Other(format!(
                        "Failed to convert geometry to gdal geometry, got err {err:?}"
                    ))
                })
            })
            .transpose()?;
        let layer = layer.layer.layer();
        let defn = layer.defn();
        let mut ft = Feature::new(defn).map_err(|err| {
            ErrorDetails::Other(format!(
                "Could not initialise feature from defn, error {err:?}"
            ))
        })?;
        if let Some(geom) = geom {
            ft.set_geometry(geom).map_err(|err| {
                ErrorDetails::Other(format!("Could not set feature geometry, error {err:?}"))
            })?;
        }
        for field in feature.fields {
            let index = defn.field_index(&field.name).unwrap();
            let val = FieldValue::from(field.value);
            ft.set_field(index, &val).map_err(|err| {
                ErrorDetails::Other(format!("Could not set field on feature, error {err:?}"))
            })?;
        }
        ft.create(layer)
            .map_err(|err| ErrorDetails::Other(format!("Failed to create feature, error {err:?}")))
    });
    state.with_lock(|state| eprintln!("{:?}", state.errors))
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
pub fn set_display_vector(state: AppState, touch_device: State<TouchDevice>) {
    state
        .with_current_vector_layer(|layer| {
            layer.info.display = true;
            touch_device.send(AppMessage::RefetchVector)
        })
        .expect("No vector found when trying to set display");
}

#[tauri::command]
#[specta::specta]
pub fn set_prefered_display_fields(
    fields: Vec<String>,
    state: AppState,
    device: State<TouchDevice>,
) {
    state.with_project(|project| {
        project.prefered_display_fields = fields;
        device.send(AppMessage::Gis(project.get_touch_device_settings()));
    });
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(tag = "option", content = "settings")]
pub enum SortOption {
    #[default]
    Default,
    Field(String),
    Area,
}

#[tauri::command]
#[specta::specta]
pub fn sort_features_by(by: SortOption, state: AppState) {
    state.with_current_vector_layer(|layer| layer.info.sort_features_by = by);
}

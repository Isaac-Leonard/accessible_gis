use std::process::Command;

use gdal::spatial_ref::SpatialRef;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumIter};

use crate::{
    state::{gis::combined::StatefulLayerEnum, AppState},
    tools::ToolDataDiscriminants,
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Deserialize, Serialize, specta::Type,))]
#[serde(tag = "type", content = "value")]
pub enum Srs {
    Proj(String),
    Wkt(String),
    Esri(String),
    Epsg(u32),
}

#[tauri::command]
#[specta::specta]
pub fn reproject_layer(srs: Srs, name: &str, state: AppState) {
    let srs = match srs {
        Srs::Proj(proj_string) => SpatialRef::from_proj4(&proj_string),
        Srs::Wkt(wkt_string) => SpatialRef::from_wkt(&wkt_string),
        Srs::Esri(esri_wkt) => SpatialRef::from_esri(&esri_wkt),
        Srs::Epsg(epsg_code) => SpatialRef::from_epsg(epsg_code),
    }
    .unwrap();
    // We've validated that the spatial reference system is valid
    let srs = srs.to_wkt().unwrap();
    state.with_current_dataset_mut(|ds, _| {
        let input_name = ds.dataset.file_name.clone();
        let layer = ds.get_current_layer();
        match layer {
            Some(StatefulLayerEnum::Vector(_layer)) => {
                let mut command = Command::new("ogr2ogr");
                command.arg("-t_srs").arg(&srs);
                command.arg(name).arg(&input_name);
                let output = command.output().unwrap();
                eprint!("{:?}", output)
            }
            Some(StatefulLayerEnum::Raster(_band)) => {
                let mut command = Command::new("gdalwarp");
                command.arg("-t_srs").arg(&srs);
                command.arg(&input_name).arg(name);
                let output = command.output().unwrap();
                eprint!("{:?}", output)
            }
            None => eprint!("No layer available"),
        }
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_srs(srs: Srs, state: AppState) {
    let srs = match srs {
        Srs::Proj(proj_string) => SpatialRef::from_proj4(&proj_string),
        Srs::Wkt(wkt_string) => SpatialRef::from_wkt(&wkt_string),
        Srs::Esri(esri_wkt) => SpatialRef::from_esri(&esri_wkt),
        Srs::Epsg(epsg_code) => SpatialRef::from_epsg(epsg_code),
    }
    .unwrap();
    state.with_current_dataset_mut(|ds, _| ds.dataset.dataset.set_spatial_ref(&srs).unwrap());
}

#[tauri::command]
#[specta::specta]
pub fn select_tool_for_current_index(tool: ToolDataDiscriminants, state: AppState) {
    state.with_lock(|state| state.shared.tools_data.add_tool(tool))
}

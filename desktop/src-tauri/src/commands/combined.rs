use gdal::spatial_ref::SpatialRef;

use crate::{
    gdal_if::Srs,
    state::{gis::combined::StatefulLayerEnum, AppState},
};

#[tauri::command]
#[specta::specta]
pub fn reproject_layer(srs: Srs, name: &str, state: AppState) {
    state.with_current_dataset_mut(|ds, _| {
        let layer = ds.get_current_layer();
        match layer {
            Some(StatefulLayerEnum::Vector(layer)) => {
                let output = layer.reproject(name, srs).unwrap();
                eprint!("{:?}", output)
            }
            Some(StatefulLayerEnum::Raster(band)) => {
                let output = band.reproject(name, srs).unwrap();
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

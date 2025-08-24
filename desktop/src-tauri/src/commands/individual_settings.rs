use tauri::{AppHandle, State};

use crate::{
    gdal_if::Srs,
    server::get_raster_path,
    state::{AppState, gis::raster::RenderMethod},
    web_socket::{AppMessage, TouchDevice},
};

#[tauri::command]
#[specta::specta]
pub fn set_current_render_method(
    render_method: RenderMethod,
    state: AppState,
    touch_device: State<TouchDevice>,
    app: AppHandle,
) {
    eprintln!("Set render method called with {render_method:?}");
    state.with_project_fallible(|project| {
        let raster_name = get_raster_path(&app, "tif");
        std::fs::remove_file(&raster_name);
        project.with_current_raster_band(|band| {
            band.info.render = render_method;
            let _band = band.reproject(&raster_name, Srs::Epsg(4326));
        });
        let wgs84_raster = project
            .datasets
            .open(raster_name, &project.settings)
            .unwrap();
        let band = wgs84_raster.get_raster(1).unwrap();
        band.info.render = render_method;
        touch_device.send(AppMessage::FetchRaster(dbg!(band.get_info_for_display())));
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_current_ocr(enabled: bool, state: AppState) {
    state
        .with_current_raster_band(|band| {
            band.info.ocr = enabled;
        })
        .expect("No raster band selected");
}

use tauri::{AppHandle, State};

use crate::{
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
    state.with_current_raster_band(|band| {
        band.info.render = render_method;
        touch_device.send(AppMessage::FetchRaster(dbg!(
            band.get_info_for_display(&app)
        )));
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

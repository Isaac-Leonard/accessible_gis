use tauri::{AppHandle, State};

use crate::{
    errors::ErrorDetails,
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
    state.with_current_raster_band_fallible(|band| -> Result<(), ErrorDetails> {
        band.info.render = render_method;
        touch_device.send(AppMessage::FetchRaster(band.get_info_for_display(&app)?));
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

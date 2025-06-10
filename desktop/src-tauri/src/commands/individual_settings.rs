use crate::state::{AppState, gis::raster::RenderMethod};

#[tauri::command]
#[specta::specta]
pub fn set_current_render_method(render_method: RenderMethod, state: AppState) {
    state
        .with_current_raster_band(|band| {
            band.info.render = render_method;
        })
        .expect("No raster band selected");
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

use std::path::PathBuf;

use tauri::AppHandle;

use crate::{
    gdal_if::processing::{aspect, roughness, slope},
    state::AppState,
    tools::{DemClassificationError, dem_to_landform_polygons},
};

macro_rules! gen_processing_command {
    ($command_name:ident, $processing_name:ident) => {
        #[tauri::command]
        #[specta::specta]
        pub fn $command_name(name: String, state: AppState) {
            state.with_lock(|state| {
                let res = state
        					.create_from_current_dataset(|ds| $processing_name(&ds.dataset, name))
        					.expect(
        						"Attempted to operate on current dataset but there is no current dataset selected",
        					);
                match res {
                    Err(e) => state.errors.push(e),
                    _ => {}
                }
            })
        }
    };
}

gen_processing_command!(calc_slope, slope);
gen_processing_command!(calc_aspect, aspect);
gen_processing_command!(calc_roughness, roughness);

#[tauri::command]
#[specta::specta]
pub fn classify_landforms(
    output: PathBuf,
    search: usize,
    threshold: f64,
    distance: usize,
    filter: usize,
    app: AppState,
    handle: AppHandle,
) -> Result<(), DemClassificationError> {
    app.with_current_raster_band(|band| {
        let input = &band.info.shared.name;
        dem_to_landform_polygons(input, &output, search, threshold, distance, filter, handle)
            .unwrap()
    })
    .unwrap();
    app.with_lock(|state| {
        state
            .open_dataset(output.to_string_lossy().to_string())
            .unwrap();
    });
    Ok(())
}

use crate::{
    gdal_if::processing::{aspect, roughness, slope},
    state::AppState,
};

macro_rules! gen_processing_command {
    ($command_name:ident, $processing_name:ident) => {
        #[tauri::command]
        #[specta::specta]
        pub fn $command_name(name: String, state: AppState) {
            state.with_lock(|state| {
                let res = state
        					.create_from_current_dataset(|ds| $processing_name(&ds.dataset, name, &Default::default()))
        					.expect(
        						"Attempted to operate on current dataset but there is no current dataset selected",
        					);
                match res {
                    Err(e) => state.errors.push(e.to_string()),
                    _ => {}
                }
            })
        }
    };
}

gen_processing_command!(calc_slope, slope);
gen_processing_command!(calc_aspect, aspect);
gen_processing_command!(calc_roughness, roughness);

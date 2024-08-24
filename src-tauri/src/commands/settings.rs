use crate::state::{settings::AudioSettings, AppState, Screen};

macro_rules! generate_general_settings_setter {
    ($fn_name:ident, $setter:ident,$type:path) => {
        #[tauri::command]
        #[specta::specta]
        pub fn $fn_name(val: $type, state: AppState) {
            eprintln!("Called settings setter");
            state.with_lock(|state| {
                state.settings.$setter(val);
            })
        }
    };
}

generate_general_settings_setter!(
    set_show_first_raster_by_default,
    set_display_first_raster,
    bool
);

generate_general_settings_setter!(
    set_show_countries_by_default,
    set_show_countries_by_default,
    bool
);

generate_general_settings_setter!(set_show_towns_by_default, set_show_towns_by_default, bool);

generate_general_settings_setter!(set_default_ocr_for_gdal, set_default_ocr_for_gdal, bool);

generate_general_settings_setter!(set_default_audio, set_default_audio, AudioSettings);

#[tauri::command]
#[specta::specta]
pub fn open_settings(state: AppState) {
    state.with_lock(|state| {
        state.screen = Screen::Settings;
    })
}

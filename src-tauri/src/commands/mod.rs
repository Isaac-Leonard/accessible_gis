mod audio;
mod combined;
mod constants;
mod context;
mod dataset;
mod dem;
mod individual_settings;
mod raster;
mod settings;
mod thiessen_polygons;
mod ui;
mod vector;

use std::path::Path;

use specta::ts::{formatter::prettier, BigIntExportBehavior, ExportConfig};
use tauri::{ipc::Invoke, Runtime, State};
use tauri_specta::{collect_commands, ts};
use tokio::sync::mpsc::UnboundedSender;

pub use crate::*;
pub use audio::*;
pub use combined::*;
pub use constants::*;
pub use context::*;
pub use dataset::*;
pub use dem::*;
pub use individual_settings::*;
pub use raster::*;
pub use settings::*;
pub use thiessen_polygons::*;
pub use ui::*;
pub use vector::*;

pub fn generate_handlers<R: Runtime>(
    s: impl AsRef<Path>,
) -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
    ts::builder()
        .commands(collect_commands![
            load_file,
            get_app_info,
            get_band_sizes,
            get_value_at_point,
            get_point_of_max_value,
            get_point_of_min_value,
            get_polygons_around_point,
            describe_line,
            describe_polygon,
            point_in_country,
            nearest_town,
            theissen_polygons_calculation,
            theissen_polygons,
            get_csv,
            theissen_polygons_to_file,
            set_screen,
            set_layer_index,
            set_dataset_index,
            set_feature_index,
            create_new_dataset,
            add_field_to_schema,
            edit_dataset,
            add_feature_to_layer,
            select_tool_for_current_index,
            get_image_pixels,
            set_name_field,
            classify_current_raster,
            set_srs,
            reproject_layer,
            copy_features,
            simplify_layer,
            message_socket,
            calc_slope,
            calc_aspect,
            calc_roughness,
            play_as_sound,
            play_histogram,
            generate_counts_report,
            open_settings,
            set_settings,
            get_render_methods,
            get_audio_indicators,
            set_display,
            set_current_ocr,
            set_current_render_method,
            set_current_audio_settings,
        ])
        .path(s)
        .config(
            ExportConfig::new()
                .bigint(BigIntExportBehavior::Number)
                .formatter(prettier),
        )
        .build()
        .unwrap()
}

#[tauri::command]
#[specta::specta]
pub fn message_socket(handle: State<UnboundedSender<()>>) {
    handle.send(()).unwrap();
}

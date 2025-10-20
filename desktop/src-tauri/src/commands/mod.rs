mod audio;
mod combined;
mod constants;
mod dataset;
mod dem;
mod individual_settings;
mod project;
mod raster;
mod settings;
mod touch_device;
mod ui;
mod vector;

use std::path::Path;

use specta_typescript::{BigIntExportBehavior, Typescript, formatter::prettier};
use tauri_specta::{Builder, collect_commands, collect_events};

pub use crate::*;
pub use audio::*;
pub use combined::*;
pub use constants::*;
pub use dataset::*;
pub use dem::*;
pub use individual_settings::*;
pub use project::*;
pub use raster::*;
pub use settings::*;
pub use touch_device::*;
pub use ui::*;
pub use vector::*;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, specta::Type, tauri_specta::Event)]
pub struct MessageEvent;

pub fn generate_handlers(s: impl AsRef<Path>) -> Builder {
    let builder = Builder::new()
        .commands(collect_commands![
            load_file,
            get_app_info,
            get_value_at_point,
            get_point_of_max_value,
            get_point_of_min_value,
            get_csv,
            set_screen,
            set_layer_index,
            set_dataset_index,
            set_feature_index,
            create_new_dataset,
            add_field_to_schema,
            edit_dataset,
            add_feature_to_layer,
            set_name_field,
            classify_current_raster,
            reproject_layer,
            copy_features,
            simplify_layer,
            calc_slope,
            calc_aspect,
            calc_roughness,
            calc_hillshade,
            calc_tpi,
            calc_tri,
            calc_color_relief,
            play_as_sound,
            play_histogram,
            generate_counts_report,
            set_settings,
            get_render_methods,
            get_wave_forms,
            set_display_raster,
            set_display_vector,
            set_current_render_method,
            set_current_audio_settings,
            focus_dataset,
            set_prefered_display_field,
            focus_box,
            classify_landforms,
            mark_error_read,
            create_project,
            load_project,
            save_project,
            toggle_labels,
            sort_features_by,
            get_landform_description,
            toggle_announce_leaving,
            toggle_announce_geometry_types,
            run_tool,
            mark_tool_output_read,
            add_custom_tool,
            get_tool_input_types,
            get_tool_preset_input_types,
            load_dataset_multi,
            add_workflow,
            run_workflow,
            get_audio_types,
            set_audio_table,
            get_esc_sounds,
            save_audio_table,
            load_audio_table,
            set_vector_line_colour,
            set_background_colour,
            get_css_colour_types,
            get_named_colours,
            set_label_colour,
            set_label_font,
            toggle_label_fill_text,
            set_label_line_width,
            set_vector_point_radius,
            set_vector_line_width,
            set_audio_point_radius,
            set_audio_line_width,
            save_tool,
            save_tools_bulk,
            load_tool,
            load_tools_bulk,
            save_workflows_bulk,
            load_workflows_bulk,
            remove_dataset,
        ])
        .events(collect_events![MessageEvent]);
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::new()
                .bigint(BigIntExportBehavior::Number)
                .formatter(prettier),
            s,
        )
        .expect("Failed to export typescript bindings");
    builder
}

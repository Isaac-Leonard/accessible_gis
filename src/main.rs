#![feature(let_chains)]
#![feature(array_windows)]
#![feature(sort_floats)]
#![feature(result_option_inspect)]
mod app;
mod audio;
mod commands;
mod events;
mod graph;
mod histogram_settings_window;
mod layout;
mod list_view;
mod new_dataset_window;
mod raster;
mod raster_graph_settings_window;
mod vector;
mod views;
mod warp;
mod windows;

use cacao::appkit::App;

fn main() {
    App::new("com.accessible.gis", app::BasicApp::default()).run();
}

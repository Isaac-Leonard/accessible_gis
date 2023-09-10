#![feature(let_chains)]
#![feature(array_windows)]
#![feature(sort_floats)]
#![feature(result_option_inspect)]
mod app;
mod audio;
mod events;
mod graph;
mod histogram_settings_window;
mod layout;
mod list_view;
mod raster;
mod vector;
mod views;
mod windows;

use cacao::{appkit::App, view::ViewController};
use views::MainView;

fn main() {
    App::new("com.accessible.gis", app::BasicApp::default()).run();
}

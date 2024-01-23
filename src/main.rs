#![feature(let_chains)]
#![feature(array_windows)]
#![feature(sort_floats)]

mod app;
mod audio;
mod commandline;
mod commands;
mod derivatives;
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

use app::BasicApp;
use cacao::appkit::App;
use clap::Parser;

use commandline::{launch_commandline_app, Input};

fn main() {
    match Input::try_parse() {
        Ok(args) => launch_commandline_app(args),
        Err(err) => {
            eprintln!("{err}");
            launch_gui_app()
        }
    };
}

fn launch_gui_app() {
    App::new("com.accessible.gis", BasicApp::default()).run();
}

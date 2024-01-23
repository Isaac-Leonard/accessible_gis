#![feature(let_chains)]
#![feature(array_windows)]
#![feature(sort_floats)]

mod audio;
mod commandline;
mod commands;
mod derivatives;
mod graph;
mod gui;
pub mod raster;
mod warp;

use cacao::appkit::App;
use clap::Parser;
use gui::app::BasicApp;

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

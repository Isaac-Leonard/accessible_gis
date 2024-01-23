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

use clap::Parser;

fn main() {
    match commandline::Input::try_parse() {
        Ok(args) => commandline::launch_commandline_app(args),
        Err(err) => {
            eprintln!("{err}");
            gui::launch_app()
        }
    };
}

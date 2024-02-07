#![feature(let_chains)]
#![feature(array_windows)]
#![feature(sort_floats)]

pub mod audio;
mod commandline;
pub mod gis;
#[cfg(target_os = "macos")]
mod gui;

use clap::Parser;

fn main() {
    match commandline::Input::try_parse() {
        Ok(args) => commandline::launch_commandline_app(args),
        Err(err) => {
            eprintln!("{err}");
            #[cfg(target_os = "macos")]
            gui::launch_app();
            #[cfg(not(target_os = "macos"))]
            exit(-1)
        }
    };
}

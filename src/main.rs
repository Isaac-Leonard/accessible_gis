#![feature(let_chains)]
#![feature(array_windows)]
#![feature(sort_floats)]
#![feature(result_option_inspect)]
mod app;
mod audio;
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

use std::{path::PathBuf, time::Duration};

use cacao::appkit::App;
use clap::{Parser, Subcommand};
use gdal::{raster::StatisticsMinMax, Dataset};
use graph::{play_rasta, RasterGraphSettings};
use raster::read_raster_data;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, global = true)]
    name: Option<PathBuf>,

    #[arg(long, default_value_t = 55.0, global = true)]
    min_freq: f64,
    #[arg(long, default_value_t = 2048.0, global = true)]
    max_freq: f64,
    #[command(subcommand)]
    command: Commands,
}

fn main2() -> bool {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            return false;
        }
    };
    let dataset = Dataset::open(&args.name.unwrap()).unwrap();
    let band = dataset.rasterband(1).unwrap();
    let data = read_raster_data(&band);
    let StatisticsMinMax { min, max } = band.compute_raster_min_max(false).unwrap();
    let no_data_value = band.no_data_value();
    let settings = RasterGraphSettings {
        min_freq: args.min_freq,
        max_freq: args.max_freq,
        row_duration: Duration::from_millis(1000),
        rows: 10,
        cols: 10,
    };
    play_rasta(data, min, max, no_data_value, settings);
    return true;
}

#[derive(Subcommand, Debug)]
enum Commands {
    graph,
}

fn main() {
    let commandline = main2();
    if commandline {
        return;
    }
    App::new("com.accessible.gis", app::BasicApp::default()).run();
}

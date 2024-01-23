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

use std::{path::PathBuf, process::exit, time::Duration};

use cacao::appkit::App;
use clap::{Args, Parser, Subcommand};
use gdal::{raster::StatisticsMinMax, Dataset};
use graph::{play_rasta, RasterGraphSettings};
use raster::read_raster_data;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Input {
    #[arg(global = true)]
    name: Option<PathBuf>,
    #[command(flatten)]
    freq_settings: FrequencyArgs,
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, default_value_t = 1, global = true)]
    band: isize,
}

#[derive(Debug, Args)]
pub struct FrequencyArgs {
    #[arg(long, default_value_t = 55.0, global = true)]
    min_freq: f64,
    #[arg(long, default_value_t = 2048.0, global = true)]
    max_freq: f64,
}

#[derive(Debug, Args)]
pub struct GraphArgs {
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    #[arg(short, long, default_value_t = 10)]
    columns: usize,
    #[arg(long, value_parser=parse_duration, default_value = "1000")]
    row_duration: Duration,
    #[arg(long, default_value_t = false)]
    classified: bool,
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    let millis = arg.parse()?;
    Ok(std::time::Duration::from_millis(millis))
}

#[derive(Subcommand, Debug)]
enum Commands {
    Graph(GraphArgs),
}

fn main() {
    match Input::try_parse() {
        Ok(args) => launch_commandline_app(args),
        Err(err) => {
            eprintln!("{err}");
            launch_gui_app()
        }
    };
}

fn launch_commandline_app(args: Input) {
    let Some(name) = args.name else {
        eprint!("No file name provided");
        exit(-1)
    };
    let Ok(dataset) = Dataset::open(&name) else {
        eprint!("Failed to read dataset at {}", name.to_string_lossy());
        exit(-1)
    };
    let Ok(band) = dataset.rasterband(args.band) else {
        eprint!(
            "Failed to read rasta band {} of the specified dataset",
            args.band
        );
        exit(-1)
    };
    let data = read_raster_data(&band);
    let Ok(StatisticsMinMax { min, max }) = band.compute_raster_min_max(false) else {
        eprint!("Could not calculate the minimum and maximum pixel values of the specified band of the dataset");
        exit(-1)
    };
    let no_data_value = band.no_data_value();
    match args.command {
        Commands::Graph(graph_settings) => {
            let settings = RasterGraphSettings {
                min_freq: args.freq_settings.min_freq,
                max_freq: args.freq_settings.max_freq,
                row_duration: graph_settings.row_duration,
                rows: graph_settings.rows,
                cols: graph_settings.columns,
                classified: graph_settings.classified,
            };
            play_rasta(data, min, max, no_data_value, settings);
        }
    }
}

fn launch_gui_app() {
    App::new("com.accessible.gis", app::BasicApp::default()).run();
}

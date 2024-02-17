use std::fmt::Display;
use std::str::FromStr;
use std::{path::PathBuf, process::exit, time::Duration};

use clap::{Args, Parser, Subcommand};
use gdal::raster::GdalDataType;
use gdal::{raster::StatisticsMinMax, Dataset};
use itertools::Itertools;

use crate::audio::graph::{play_rasta, RasterGraphSettings};
use crate::audio::{
    histogram::{generate_image_histogram, play_histogram},
    Waveform,
};
use crate::gis::raster::read_raster_data;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Input {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
pub struct HistogramArgs {
    #[arg(global = true)]
    name: Option<PathBuf>,
    #[arg(short, long, default_value_t = 1, global = true)]
    band: isize,
    #[command(flatten)]
    freq_settings: FrequencyArgs,
    #[arg(short, long, default_value_t=WaveType::Sine, global = true)]
    wave: WaveType,
}

#[derive(Debug, Args, Clone)]
pub struct FrequencyArgs {
    #[arg(long, default_value_t = 55.0)]
    min_freq: f64,
    #[arg(long, default_value_t = 2048.0)]
    max_freq: f64,
}

#[derive(Debug, Args, Clone)]
pub struct GlobalGraphArgs {
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    #[arg(short, long, default_value_t = 10)]
    columns: usize,
    #[arg(long, value_parser=parse_duration, default_value = "1000")]
    row_duration: Duration,
    #[arg(long, default_value_t = false)]
    classified: bool,
}

#[derive(Debug, Args, Clone)]
pub struct IndividualGraphArgs {
    #[arg()]
    pub name: String,
    #[arg(short, long, default_value_t=WaveType::Sine)]
    pub wave: WaveType,
    #[arg(short, long, default_value_t = 1)]
    pub band: isize,
    #[command(flatten)]
    pub freq_settings: FrequencyArgs,
    #[command(flatten)]
    pub global: GlobalGraphArgs,
}

#[derive(Debug, Args)]
pub struct MultiBandArgs {
    #[arg(global = true)]
    pub band: BandWave,
}

#[derive(Debug, Clone)]
pub struct BandWave(usize, WaveType);
impl FromStr for BandWave {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((band, wave)) = s.split('=').map(str::trim).collect_tuple::<(_, _)>() else {
            return Err("Invalid band=wave specified");
        };
        Ok(Self(
            band.parse()
                .map_err(|_| "Band must be an integer in band wave pairs")?,
            wave.parse()?,
        ))
    }
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    let millis = arg.parse()?;
    Ok(std::time::Duration::from_millis(millis))
}

#[derive(Subcommand, Debug)]
enum Commands {
    Graph(IndividualGraphArgs),
    Histogram(HistogramArgs),
}

#[derive(Debug, Args)]
pub struct GraphArgs {
    #[command(flatten)]
    graph1: IndividualGraphArgs,
    #[command(flatten)]
    graph2: Option<IndividualGraphArgs>,
    #[command(flatten)]
    graph3: Option<IndividualGraphArgs>,
}

pub fn launch_commandline_app(args: Input) {
    match args.command {
        Commands::Graph(args) => {
            let args = args.clone();
            let name = args.name;
            let Ok(dataset) = Dataset::open(&name) else {
                eprint!("Failed to read dataset at {}", name);
                exit(-1)
            };
            let Ok(band) = dataset.rasterband(args.band) else {
                eprint!(
                    "Failed to read rasta band {} of the specified dataset",
                    args.band
                );
                exit(-1)
            };
            let wave: Waveform = args.wave.into();
            let data = read_raster_data(&band);
            let Ok(StatisticsMinMax { min, max }) = band.compute_raster_min_max(false) else {
                eprint!("Could not calculate the minimum and maximum pixel values of the specified band of the dataset");
                exit(-1)
            };
            let no_data_value = band.no_data_value();
            let settings = RasterGraphSettings {
                min_freq: args.freq_settings.min_freq,
                max_freq: args.freq_settings.max_freq,
                row_duration: args.global.row_duration,
                rows: args.global.rows,
                cols: args.global.columns,
                classified: args.global.classified,
                wave,
            };
            play_rasta(data, min, max, no_data_value, settings);
        }
        Commands::Histogram(args) => {
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
            let wave: Waveform = args.wave.into();
            let data = match band.band_type() {
                GdalDataType::UInt8 => band
                    .read_as_array::<u8>((0, 0), band.size(), band.size(), None)
                    .unwrap(),
                _ => {
                    eprint!("Currently we can only generate histograms for byte data");
                    exit(-1);
                }
            };
            let counts = generate_image_histogram(data.into_raw_vec());
            play_histogram(counts, Default::default(), wave);
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum WaveType {
    #[default]
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

impl FromStr for WaveType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sine" => Ok(Self::Sine),
            "triangle" => Ok(Self::Triangle),
            "square" => Ok(Self::Square),
            "sawtooth" => Ok(Self::Sawtooth),
            _ => Err("Not a valid wave type"),
        }
    }
}

impl Display for WaveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sine => f.write_str("sine"),
            Self::Triangle => f.write_str("triangle"),
            Self::Square => f.write_str("square"),
            Self::Sawtooth => f.write_str("sawtooth"),
        }
    }
}

impl From<WaveType> for Waveform {
    fn from(value: WaveType) -> Self {
        match value {
            WaveType::Sine => Self::Sine,
            WaveType::Triangle => Self::Triangle,
            WaveType::Square => Self::Square,
            WaveType::Sawtooth => Self::Sawtooth,
        }
    }
}

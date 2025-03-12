use std::fmt::Display;
use std::str::FromStr;
use std::{path::PathBuf, process::exit, time::Duration};

use clap::{Args, Parser, Subcommand};
use gdal::{raster::StatisticsMinMax, Dataset};
use itertools::Itertools;
use ndarray::Array2;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};

use crate::audio::graph::{play_rasta, RasterGraphSettings};
use crate::audio::{histogram::play_histogram, Waveform};
use crate::gdal_if::read_raster_data;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Input {
    #[command(subcommand)]
    command: AllCommands,
}

#[derive(Debug, Args, Clone, Serialize, Deserialize)]
pub struct HistogramArgs {
    #[arg()]
    #[serde(default)]
    name: PathBuf,
    #[arg(short, long, default_value_t = 1, global = true)]
    #[serde(default = "default_band")]
    band: usize,
    #[command(flatten)]
    #[serde(flatten)]
    freq_settings: FrequencyArgs,
    #[arg(short, long, default_value_t=WaveType::Sine, global = true)]
    #[serde(default)]
    wave: WaveType,
}

fn default_band() -> usize {
    1
}

#[derive(Debug, Args, Clone, Serialize, Deserialize)]
pub struct FrequencyArgs {
    #[arg(long, default_value_t = 55.0)]
    #[serde(default = "default_min_freq")]
    min_freq: f64,
    #[arg(long, default_value_t = 2048.0)]
    #[serde(default = "default_max_freq")]
    max_freq: f64,
}

fn default_min_freq() -> f64 {
    55.0
}

fn default_max_freq() -> f64 {
    2048.0
}

#[derive(Debug, Args, Clone, Serialize, Deserialize)]
pub struct GlobalGraphArgs {
    #[arg(short, long, default_value_t = 10)]
    #[serde(default = "default_rows")]
    rows: usize,
    #[arg(short, long, default_value_t = 10)]
    #[serde(default = "default_cols")]
    columns: usize,
    #[arg(long, value_parser=parse_duration, default_value = "1000")]
    #[serde(
        default = "default_row_duration",
        deserialize_with = "deserialize_duration"
    )]
    row_duration: Duration,
    #[arg(long, default_value_t = false)]
    #[serde(default)]
    classified: bool,
}
fn default_rows() -> usize {
    10
}
fn default_cols() -> usize {
    10
}
fn default_row_duration() -> Duration {
    Duration::from_millis(1000)
}

#[derive(Debug, Args, Clone, Serialize, Deserialize)]
pub struct IndividualGraphArgs {
    #[arg()]
    pub name: String,
    #[arg(short, long, default_value_t=WaveType::Sine)]
    #[serde(default)]
    pub wave: WaveType,
    #[arg(short, long, default_value_t = 1)]
    #[serde(default = "default_band")]
    pub band: usize,
    #[command(flatten)]
    #[serde(flatten)]
    pub freq_settings: FrequencyArgs,
    #[command(flatten)]
    #[serde(flatten)]
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
enum AllCommands {
    Json(JsonArgs),
    #[command(flatten)]
    Commands(Commands),
}

#[derive(Subcommand, Debug)]
enum Commands {
    Graph(IndividualGraphArgs),
    Histogram(HistogramArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum JsonCommands {
    Graph(Vec<IndividualGraphArgs>),
    Histogram(HistogramArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct JsonArgs {
    #[arg()]
    payload: JsonCommands,
}

impl FromStr for JsonCommands {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
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
        AllCommands::Commands(cmd) => match cmd {
            Commands::Graph(args) => run_single_graph(args),
            Commands::Histogram(args) => run_histogram(args),
        },
        AllCommands::Json(JsonArgs { payload }) => match payload {
            JsonCommands::Histogram(args) => run_histogram(args),
            JsonCommands::Graph(args) => run_multiple_graph(args),
        },
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

fn run_histogram(args: HistogramArgs) {
    let name = args.name;
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
    let Ok(StatisticsMinMax { min, max }) = band.compute_raster_min_max(false) else {
        eprint!("Failed to get min max of raster");
        exit(-1);
    };
    let Ok(histogram) = band.histogram(min, max, 256, true, false) else {
        eprintln!("Failed to get histogram of raster");
        exit(-1);
    };
    let wave: Waveform = args.wave.into();
    let counts = histogram.counts().iter().map(|x| (*x) as f64).collect_vec();
    play_histogram(counts, Default::default(), wave);
}

fn gen_graph_options(
    args: IndividualGraphArgs,
) -> (Array2<f64>, f64, f64, Option<f64>, RasterGraphSettings) {
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
        min_value: None,
        max_value: None,
    };
    (data, min, max, no_data_value, settings)
}

fn run_single_graph(args: IndividualGraphArgs) {
    let vals = gen_graph_options(args);
    play_rasta(vec![vals]);
}
fn run_multiple_graph(args: Vec<IndividualGraphArgs>) {
    let vals = args.into_iter().map(gen_graph_options).collect();
    play_rasta(vals);
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_u64(MillisVisitor)
}

struct MillisVisitor;
impl<'de> Visitor<'de> for MillisVisitor {
    type Value = Duration;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A u64 value representing milliseconds")
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Duration::from_millis(v))
    }
}

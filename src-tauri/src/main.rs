// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(try_blocks)]
#![feature(iter_map_windows)]

mod audio;
mod commandline;
mod commands;
mod files;
mod gdal_if;
mod geometry;
mod math;
mod server;
mod state;
mod stats;
mod tools;
mod ui;
mod web_socket;

use geo_types::Polygon;
pub use state::dataset_collection;

use clap::Parser;
use files::get_csv;
use gdal::{vector::LayerAccess, Dataset};
use gdal_if::{Field, LayerIndex};
use geometry::Geometry;
use local_ip_address::local_ip;
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};
use state::AppData;
use tauri::Manager;

use std::sync::{Arc, Mutex};

use crate::{
    audio::get_audio,
    commands::generate_handlers,
    gdal_if::LocalFeatureInfo,
    server::run_server,
    state::{AppDataSync, Country, PreloadedAppData},
};

fn main() {
    match commandline::Input::try_parse() {
        Ok(args) => commandline::launch_commandline_app(args),
        Err(err) => {
            eprintln!("{err}");
            launch_gui();
        }
    };
}

fn launch_gui() {
    let handlers = generate_handlers("../src/bindings.ts");
    let countries = load_countries();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppDataSync {
            data: Arc::new(Mutex::new(AppData::new())),
            default_data: PreloadedAppData { countries },
        })
        .invoke_handler(handlers)
        .setup(|app| {
            //            let window = app.get_webview_window("main").unwrap();
            //            window.open_devtools();
            let audio = get_audio();
            app.manage(audio);
            let state = (*app.state::<AppDataSync>()).clone();
            let handle = app.handle();
            handle.manage(tauri::async_runtime::spawn(run_server(
                state.clone(),
                handle.clone(),
            )));
            match local_ip() {
                Ok(local_ip) => {
                    // Print the IP address and port
                    let port = 80;
                    println!("Server running at http://{}:{}/", local_ip, port);
                    println!("cwd {:?}", std::env::current_dir());
                }
                Err(error) => println!("Unable to retrieve local IP address, got error: {}", error),
            };

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/*
// Use column major ordering for this array
// So index with cell[x][y]
fn describe_3x3_cell(cell: [[f64; 3]; 3]) -> String {
    let midpoint_val = cell[1][1];
    let indexed_iter = cell
        .into_iter()
        .enumerate()
        .flat_map(|(y, row)| row.into_iter().enumerate().map(|(x, v)| ((x, y), v)))
        .collect::<Vec<_>>();
}

fn is_corner_of_3x3(p: (usize, usize)) -> bool {
    match p {
        (0, 0) => true,
        (0, 2) => true,
        (2, 0) => true,
        (2, 2) => true,
        _ => false,
    }
}

fn is_left_of_3x3(p: (usize, usize)) -> bool {
    p.1 == 0
}

fn is_center_of_3x3(p: (usize, usize)) -> bool {
    p == (1, 1)
}
*/

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, specta::Type)]
pub struct LayerDescriptor {
    dataset: usize,
    #[serde(flatten)]
    band: LayerIndex,
    dataset_file: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, specta::Type)]
#[serde(tag = "type")]
pub enum LayerInfo {
    Raster { width: usize, length: usize },
    Vector,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FeatureInfo {
    fields: Vec<Field>,
    geometry: Option<Geometry>,
    fid: Option<u64>,
}

impl FeatureInfo {
    fn new(geometry: Geometry, fields: Vec<Field>) -> Self {
        Self {
            geometry: Some(geometry),
            fields,
            fid: None,
        }
    }
}

fn load_countries() -> RTree<GeomWithData<Polygon, Vec<Field>>> {
    let countries_path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("data/countries.geojson");
    let countries = RTree::bulk_load(
        Dataset::open(countries_path)
            .unwrap()
            .layer(0)
            .unwrap()
            .features()
            .flat_map(|feature| {
                Vec::<Country>::from(LocalFeatureInfo {
                    geometry: feature.geometry().unwrap().to_geo().unwrap(),
                    fields: feature.fields().map(Into::into).collect(),
                })
            })
            .collect::<Vec<_>>(),
    );
    countries
}

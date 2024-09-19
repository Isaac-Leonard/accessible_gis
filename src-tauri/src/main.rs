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

use commands::web_socket::TouchDevice;
use geo_types::Polygon;
pub use state::dataset_collection;

use clap::Parser;
use files::get_csv;
use gdal::{vector::LayerAccess, Dataset};
use gdal_if::Field;
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
    let _ = fix_path_env::fix();
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
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(TouchDevice::default())
        .invoke_handler(handlers)
        .setup(|app| {
            let resolver = app.path();
            let countries = load_countries(resolver);
            app.manage(AppDataSync {
                data: Arc::new(Mutex::new(AppData::new(resolver))),
                default_data: PreloadedAppData { countries },
            });

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

fn load_countries<R: tauri::Runtime>(
    resolver: &tauri::path::PathResolver<R>,
) -> RTree<GeomWithData<Polygon, Vec<Field>>> {
    let countries_path = resolver
        .resolve(
            "data/countries.geojson",
            tauri::path::BaseDirectory::Resource,
        )
        .unwrap();
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

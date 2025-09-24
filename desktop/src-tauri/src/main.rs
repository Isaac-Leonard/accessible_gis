// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod commandline;
mod commands;
mod errors;
mod files;
mod gdal_if;
mod geometry;
mod server;
mod state;
mod tools;
mod ui;
mod utils;
mod web_socket;

use commands::web_socket::TouchDevice;
use geo_types::Polygon;
pub use state::dataset_collection;

use clap::Parser;
use files::get_csv;
use gdal::{Dataset, vector::LayerAccess};
use gdal_if::Field;
use geometry::Geometry;
use rstar::{RTree, primitives::GeomWithData};
use serde::{Deserialize, Serialize};
use state::{AppData, gis::raster::load_esc_sounds};
use tauri::{AppHandle, Manager};

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
    let specta_builder = generate_handlers("../src/bindings.ts");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(TouchDevice::default())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            std::fs::create_dir_all(app.path().temp_dir().unwrap()).unwrap();
            let countries = load_countries(app.handle());
            let esc_sounds = load_esc_sounds(app.handle()).unwrap();
            app.manage(AppDataSync {
                data: Arc::new(Mutex::new(AppData::new(app.handle()))),
                default_data: PreloadedAppData {
                    countries,
                    esc_sounds,
                },
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
    pub fn new(geometry: Geometry, fields: Vec<Field>) -> Self {
        Self {
            geometry: Some(geometry),
            fields,
            fid: None,
        }
    }
}

fn load_countries(app: &AppHandle) -> RTree<GeomWithData<Polygon, Vec<Field>>> {
    let countries_path = app
        .path()
        .resolve(
            "data/countries.geojson",
            tauri::path::BaseDirectory::Resource,
        )
        .unwrap();
    eprintln!("Countries path: {:?}", countries_path);
    let countries = RTree::bulk_load(
        Dataset::open(countries_path)
            .unwrap()
            .layer(0)
            .unwrap()
            .features()
            .flat_map(|feature| {
                Vec::<Country>::from(LocalFeatureInfo {
                    geometry: feature
                        .geometry()
                        .map(|geom| geom.to_geo())
                        .transpose()
                        .unwrap(),
                    fields: feature.fields().map(Into::into).collect(),
                })
            })
            .collect::<Vec<_>>(),
    );
    countries
}

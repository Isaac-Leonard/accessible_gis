use std::{io::Write, path::PathBuf};

use actix_files::{self as fs};
use actix_web::{
    get,
    http::header::ContentType,
    web::{self, Data, Json, PayloadConfig},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tokio::task::spawn_local;

use crate::{
    gdal_if::{merge_layers, read_raster_data_enum, Srs},
    state::AppDataSync,
    web_socket::ws_handle,
};

fn get_raster_path(app: &AppHandle) -> PathBuf {
    app.path()
        .resolve("raster.tif", BaseDirectory::Temp)
        .unwrap()
}

#[get("/get_raster")]
async fn get_raster(state: Data<AppDataSync>, app: Data<AppHandle>) -> impl Responder {
    eprintln!("get_raster called");
    let raster_name = get_raster_path(&app);
    let Some((metadata, data)) = state.with_lock(|state| -> Option<(_, _)> {
        let output = state
            .shared
            .get_raster_to_display()?
            .reproject(&raster_name, Srs::Epsg(4326));
        eprintln!("{:?}", output);
        let wgs84_raster = state
            .open_dataset(raster_name.to_str().unwrap().to_string())
            .unwrap();
        let band = wgs84_raster.get_raster(1).unwrap();
        let data = read_raster_data_enum(&band.band.band)?;
        let metadata = band.get_info_for_display();
        Some((metadata, data))
    }) else {
        return HttpResponse::NotFound().finish();
    };
    eprintln!("Metadata: {:?}", metadata);

    let mut bytes = Vec::<u8>::new();
    bytes
        .write_all(metadata.resolution.to_le_bytes().as_slice())
        .unwrap();
    bytes
        .write_all(metadata.width.to_le_bytes().as_slice())
        .unwrap();
    bytes
        .write_all(metadata.height.to_le_bytes().as_slice())
        .unwrap();
    bytes
        .write_all(metadata.origin.0.to_le_bytes().as_slice())
        .unwrap();
    bytes
        .write_all(metadata.origin.1.to_le_bytes().as_slice())
        .unwrap();
    data.into_f64_vec().into_iter().for_each(|x| {
        if let Err(e) = bytes.write_all((x as f32).to_le_bytes().as_slice()) {
            panic!("Got error when writing response: {:?}", e)
        }
    });
    HttpResponse::Ok().body(bytes)
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct ImageSize {
    pub width: usize,
    pub height: usize,
    pub bands: Option<usize>,
}

pub async fn run_server(state: AppDataSync, app_handle: AppHandle) {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .app_data(Data::new(app_handle.clone()))
            .app_data(Data::new(PayloadConfig::new(1024 * 1024 * 1024)))
            .service(get_raster)
            .service(get_info)
            .service(get_ocr)
            .service(get_vector)
            .service(web::resource("/ws").route(web::get().to(ws)))
            .service(
                fs::Files::new(
                    "/",
                    app_handle
                        .path()
                        .resolve("external-touch-device/", BaseDirectory::Resource)
                        .unwrap(),
                )
                .show_files_listing()
                .index_file("index.html"),
            )
    })
    .bind(("0.0.0.0", 80))
    .unwrap()
    .run()
    .await
    .unwrap();
}

#[get("/get_vector")]
async fn get_vector(app: Data<AppHandle>) -> impl Responder {
    std::fs::create_dir_all(app.path().temp_dir().unwrap()).unwrap();
    eprintln!("get_vector called");
    let json_name = app
        .path()
        .resolve("vector.json", BaseDirectory::Temp)
        .unwrap();
    let state = app.state::<AppDataSync>();
    let succeeded = state.with_lock(|state| {
        let layers = state.shared.get_vectors_for_display();
        if layers.is_empty() {
            return false;
        }
        let layer_names = layers
            .into_iter()
            .map(|layer| layer.info.shared.name.clone())
            .dedup()
            .collect_vec();
        let output = merge_layers(layer_names, true, Srs::Epsg(4326), &json_name, true).unwrap();
        eprintln!("{:?}", output);
        true
    });
    if succeeded {
        let data = std::fs::read(json_name).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(String::from_utf8_lossy(&data).to_string())
    } else {
        // Send empty array there is no vector layer
        HttpResponse::Ok().json(json! ({
            "type":"FeatureCollection", "features":[]
        }))
    }
}

/// Handshake and start WebSocket handler with heartbeats.
async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    app_handle: web::Data<AppHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(ws_handle((**app_handle).clone(), session, msg_stream));
    Ok(res)
}

#[get("/get_info")]
async fn get_info(state: Data<AppDataSync>) -> impl Responder {
    Json(state.with_lock(|state| Some(state.shared.get_raster_to_display()?.info.render)))
}

#[get("/get_ocr")]
async fn get_ocr(state: Data<AppDataSync>) -> impl Responder {
    Json(state.with_lock(|state| Some(state.shared.get_raster_to_display()?.info.ocr)))
}

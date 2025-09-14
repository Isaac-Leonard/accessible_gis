use actix_files::{self as fs};
use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get,
    http::header::ContentType,
    web::{self, Data, Json, Path, PayloadConfig},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager, path::BaseDirectory};
use tokio::task::spawn_local;

use crate::{
    errors::ErrorDetails,
    files::get_random_temp_path,
    gdal_if::{Srs, merge_layers, read_raster_data_enum},
    state::AppDataSync,
    web_socket::ws_handle,
};

#[get("/get_raster")]
async fn get_raster(state: Data<AppDataSync>) -> impl Responder {
    eprintln!("get_raster called");
    let Some(data) = state.with_project_fallible(|project| {
        let band_to_display = project
            .get_raster_to_display()
            .ok_or_else(|| ErrorDetails::Other("No raster to display".to_string()))?;
        let index = band_to_display.get_index();
        let dataset = band_to_display
            .info
            .wgs84_reprojected_file
            .as_mut()
            .ok_or_else(|| {
                ErrorDetails::Other("No reprojected dataset for display raster".to_string())
            })?;
        let band = dataset.get_raster(index).ok_or_else(|| {
            ErrorDetails::Other("Failed to get band for reprojected display raster".to_string())
        })?;
        read_raster_data_enum(&band.band).ok_or_else(|| {
            ErrorDetails::Other(
                "Failed to read data for reprojected version of display raster".to_string(),
            )
        })
    }) else {
        return HttpResponse::NotFound().finish();
    };
    let bytes = data
        .into_f64_vec()
        .into_iter()
        .flat_map(|x| x.to_le_bytes())
        .collect_vec();
    eprintln!("Sending {}Mb to touch device", bytes.len() / 1024 / 1024);
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
            .service(get_image)
            .service(get_audio)
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
    let succeeded = state
        .with_project_fallible(|project| {
            let layers = project.get_vectors_for_display();
            let layer_names = layers
                .into_iter()
                .map(|layer| layer.info.shared.name.clone())
                .dedup()
                .collect_vec();
            if layer_names.is_empty() {
                return Ok(false);
            }

            let output = merge_layers(layer_names, true, Srs::Epsg(4326), &json_name, true)?;
            eprintln!("{output:?}");
            Ok(true)
        })
        .is_some_and(|b| b);
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
    Json(state.with_lock(|state| {
        state.with_project(|project| {
            project
                .get_raster_to_display()
                .map(|raster| raster.info.render)
        })?
    }))
}

#[get("/get_ocr")]
async fn get_ocr(state: Data<AppDataSync>) -> impl Responder {
    Json(state.with_lock(|state| {
        state.with_project(|project| Some(project.get_raster_to_display()?.info.render))
    }))
}

#[get("/get_image")]
async fn get_image(state: Data<AppDataSync>, app: Data<AppHandle>) -> impl Responder {
    let raster_name = get_random_temp_path(&app, "png");
    // Deliberately ignore the result as it is almost certain to get an error as most paths should be unique.
    let _ = std::fs::remove_file(&raster_name);
    state.with_lock(|state| {
        let output = state.with_project(|project| {
            project
                .get_raster_to_display()
                .map(|raster| dbg!(raster.reproject(&raster_name, Srs::Epsg(4326))))
        });
        eprintln!("{:?}", output);
    });
    fs::NamedFile::open_async(raster_name).await
}

#[get("/get_audio/{index}.wav")]
async fn get_audio(
    index: Path<usize>,
    state: Data<AppDataSync>,
    app: Data<AppHandle>,
) -> impl Responder {
    dbg!(&index);
    let filename = &state.default_data.esc_sounds[*index].filename;
    const ESC_AUDIO_DIR: &str = "esc-50/audio/";

    let path = app
        .path()
        .resolve(
            format!("{ESC_AUDIO_DIR}/{filename}"),
            tauri::path::BaseDirectory::Resource,
        )
        .unwrap();
    dbg!(&path);
    fs::NamedFile::open_async(path).await
}

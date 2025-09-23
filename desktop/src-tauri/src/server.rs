use actix_files::{self as fs};
use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get,
    http::header::ContentType,
    mime,
    web::{self, Data, Path, PayloadConfig},
};
use itertools::Itertools;
use tauri::{AppHandle, Manager, path::BaseDirectory};
use tokio::task::spawn_local;

use crate::{
    errors::ErrorDetails,
    files::get_random_temp_path,
    gdal_if::{Srs, merge_layers, read_raster_data_enum},
    state::AppDataSync,
    web_socket::ws_handle,
};

pub async fn run_server(state: AppDataSync, app_handle: AppHandle) {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .app_data(Data::new(app_handle.clone()))
            .app_data(Data::new(PayloadConfig::new(1024 * 1024 * 1024)))
            .service(get_raster)
            .service(get_image)
            .service(get_audio)
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

#[get("/get_vector/{name}")]
async fn get_vector(
    name: Path<String>,
    app: Data<AppHandle>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    eprintln!("get_vector called: {name}");

    let json_name = get_random_temp_path(&app, "geojson");
    let state = app.state::<AppDataSync>();

    let found_layer = state.with_project_fallible(|project| {
        let layers = project.get_vectors_for_display();
        layers
            .into_iter()
            .find(|layer| layer.get_touch_device_layer_name() == *name)
            .map(|layer| layer.info.shared.name.clone())
            .ok_or_else(|| {
                ErrorDetails::Other("Could not find vector layer for touch device".to_string())
            })
    });

    if let Some(layer_name) = found_layer {
        if let Err(err) = merge_layers(vec![layer_name], true, Srs::Epsg(4326), &json_name, true) {
            state.with_lock(|state| state.errors.push(err.into()));
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    Ok(fs::NamedFile::open_async(&json_name)
        .await?
        .set_content_type(mime::APPLICATION_JSON)
        .disable_content_disposition()
        .into_response(&req))
}

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
        read_raster_data_enum(band.band()).ok_or_else(|| {
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

#[get("/get_image")]
async fn get_image(state: Data<AppDataSync>, app: Data<AppHandle>) -> impl Responder {
    let raster_name = get_random_temp_path(&app, "png");
    // Deliberately ignore the result as it is almost certain to get an error as most paths should be unique.
    let _ = std::fs::remove_file(&raster_name);
    state.with_lock(|state| {
        let _output = state.with_project(|project| {
            project
                .get_raster_to_display()
                .map(|raster| dbg!(raster.reproject(&raster_name, Srs::Epsg(4326))))
        });
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

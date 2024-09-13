use actix_files::{self as fs};
use actix_web::{
    get,
    http::header::ContentType,
    web::{self, Data, Json},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use gdal::Dataset;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::task::spawn_local;

use crate::{
    gdal_if::{merge_layers, read_raster_data_enum, RasterData, Srs},
    state::AppDataSync,
    web_socket::ws_handle,
};

#[get("/get_raster")]
async fn get_raster(state: Data<AppDataSync>) -> impl Responder {
    eprintln!("get_raster called");
    let raster_name = "../data/raster.tif";
    let data = state.with_lock(|state| -> Option<_> {
        let output = state
            .shared
            .get_raster_to_display()?
            .reproject(raster_name, Srs::Epsg(4326));
        eprintln!("{:?}", output);
        let wgs84_raster = Dataset::open(raster_name).unwrap();
        let band = wgs84_raster.rasterband(1).unwrap();
        read_raster_data_enum(&band)
    });
    match data {
        Some(data) => HttpResponse::Ok().body(
            data.to_f64()
                .into_iter()
                .flat_map(|x| (x as f32).to_le_bytes())
                .collect_vec(),
        ),
        None => HttpResponse::NotFound().finish(),
    }
}

#[derive(Serialize)]
struct ImageData {
    pub width: usize,
    pub height: usize,
    #[serde(flatten)]
    pub data: RasterData,
    pub no_data_value: Option<f64>,
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
            .service(get_raster)
            .service(get_info)
            .service(get_ocr)
            .service(get_vector)
            .service(get_raster_meta)
            .service(web::resource("/ws").route(web::get().to(ws)))
            .service(
                fs::Files::new("/", "../external-touch-device/dist/")
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
    eprintln!("get_vector called");
    let json_name = "../vector.json";
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
        let output = merge_layers(layer_names, true, Srs::Epsg(4326), json_name, true).unwrap();
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
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("[]")
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
    state.with_lock(|state| {
        Json::<Option<_>>(try { state.shared.get_raster_to_display()?.info.render })
    })
}

#[get("/get_raster_meta")]
async fn get_raster_meta(state: Data<AppDataSync>) -> impl Responder {
    state.with_lock(|state| {
        Json::<Option<_>>(try { state.shared.get_raster_to_display()?.get_info_for_display() })
    })
}

#[get("/get_ocr")]
async fn get_ocr(state: Data<AppDataSync>) -> impl Responder {
    state.with_lock(|state| {
        Json::<Option<_>>(try { state.shared.get_raster_to_display()?.info.ocr })
    })
}

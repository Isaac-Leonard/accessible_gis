use actix_files::{self as fs, NamedFile};
use actix_web::{
    get,
    web::{self, Data, Json, Query},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::task::spawn_local;

use crate::{
    gdal_if::{read_raster_data_enum_as, RasterData},
    state::AppDataSync,
    web_socket::ws_handle,
    FeatureInfo,
};

#[get("/get_image")]
async fn get_image(size: Query<ImageSize>, state: Data<AppDataSync>) -> impl Responder {
    let ImageSize { width, height, .. } = *size;
    let (no_data_value, data) = state
        .with_current_raster_band(|band| {
            (
                band.band.band().no_data_value(),
                read_raster_data_enum_as(
                    band.band.band(),
                    (0, 0),
                    band.band.band().size(),
                    (size.width, size.height),
                    None,
                )
                .unwrap(),
            )
        })
        .ok_or_else(|| "Couldn't read band data".to_owned())
        .unwrap();
    Json(ImageData {
        width,
        height,
        data,
        no_data_value,
    })
}

#[get("/get_file")]
async fn get_file(state: Data<AppDataSync>) -> impl Responder {
    let name = state
        .with_current_dataset_mut(|ds, _| ds.dataset.file_name.clone())
        .ok_or_else(|| "Couldn't get raster file name".to_owned())
        .unwrap();
    NamedFile::open(name).unwrap()
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
            .service(get_image)
            .service(get_file)
            .service(web::resource("/ws").route(web::get().to(ws)))
            .service(
                fs::Files::new("/", "../static/dist/")
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

#[get("/get_image")]
async fn get_vector(state: Data<AppDataSync>) -> Json<Vec<FeatureInfo>> {
    Json(state.with_lock(|state| state.shared.get_vectors_for_display()))
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

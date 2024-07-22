use actix_files::{self as fs, NamedFile};
use actix_web::{
    dev::Server,
    get,
    web::{Data, Json, Query},
    App, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};

use crate::{
    gdal_if::{read_raster_data_enum_as, RasterData},
    state::AppDataSync,
    FeatureInfo,
};

#[get("/get_image")]
async fn get_image(size: Query<ImageSize>, state: Data<AppDataSync>) -> impl Responder {
    let ImageSize { width, height, .. } = *size;
    let (no_data_value, data) = state
        .with_current_raster_band(|band| {
            (
                band.band().no_data_value(),
                read_raster_data_enum_as(
                    band.band(),
                    (0, 0),
                    band.band().size(),
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

pub async fn run_server(state: AppDataSync) -> std::io::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .service(get_image)
            .service(get_file)
            .service(
                fs::Files::new("/", "../static/dist/")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .bind(("0.0.0.0", 80))?
    .run())
}

#[get("/get_image")]
async fn get_vector(state: Data<AppDataSync>) -> Json<Vec<FeatureInfo>> {
    Json(state.with_lock(|state| state.shared.get_vectors_for_display()))
}

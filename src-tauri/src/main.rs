// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(try_blocks)]
#![feature(iter_map_windows)]

mod dataset_collection;
mod files;
mod gdal_if;
mod geometry;
mod math;
mod server;
mod state;
mod stats;
mod thiessen_polygons;
mod tools;
mod ui;

use core::cmp::Ordering;
use dataset_collection::{StatefulDataset, StatefulLayerEnum, StatefulVectorInfo};
use files::get_csv;
use gdal::{
    spatial_ref::SpatialRef,
    vector::{LayerAccess, ToGdal},
    Dataset, DatasetOptions, GdalOpenFlags,
};
use gdal_if::{
    list_drivers, read_raster_data, read_raster_data_enum_as, Field, FieldType, LayerIndex,
    WrappedDataset,
};
use geo::{
    Area, ChamberlainDuquetteArea, Closest, ClosestPoint, Contains, GeodesicArea, GeodesicBearing,
    GeodesicDistance, GeodesicLength, Geometry as GeoGeometry, Intersects, Point as GeoPoint,
    Within,
};
use geo_types::{LineString as GeoLineString, Polygon as GeoPolygon};
use geometry::{Geometry, LineString, Point, Polygon, SingleGeometry, ToGeometryType};
use itertools::Itertools;
use local_ip_address::local_ip;
use proj::Transform;
use rstar::{RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use specta::ts::{formatter::prettier, BigIntExportBehavior, ExportConfig};
use state::{AppData, AppState, Screen};
use statrs::statistics::Statistics;
use strum::{EnumDiscriminants, EnumIter};
use tauri::{ipc::Invoke, Manager, Runtime};
use tauri_specta::{collect_commands, ts};
use tools::ToolDataDiscriminants;
use ui::{NewDatasetScreenData, UiScreen};

use std::{path::Path, process::Command, sync::MutexGuard};

use crate::{
    gdal_if::LocalFeatureInfo,
    server::run_server,
    state::{AppDataSync, Country, CountryImpl, PreloadedAppData},
    thiessen_polygons::{
        theissen_polygons, theissen_polygons_calculation, theissen_polygons_to_file,
    },
};

#[tauri::command]
#[specta::specta]
fn load_file(name: String, state: AppState) -> Result<(), String> {
    let mut guard = state.data.lock().unwrap();
    guard.shared.datasets.open(name).map(|_| ())
}

#[tauri::command]
#[specta::specta]
fn get_app_info(state: AppState) -> UiScreen {
    state.with_lock(|state| match state.screen {
        Screen::Main => UiScreen::Layers(state.get_layers_screen()),
        Screen::NewDataset => UiScreen::NewDataset(NewDatasetScreenData {
            drivers: list_drivers(),
        }),
    })
}

fn generate_handlers<R: Runtime>(
    s: impl AsRef<Path>,
) -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
    ts::builder()
        .commands(collect_commands![
            load_file,
            get_app_info,
            get_band_sizes,
            get_value_at_point,
            get_point_of_max_value,
            get_point_of_min_value,
            get_polygons_around_point,
            describe_line,
            describe_polygon,
            point_in_country,
            nearest_town,
            theissen_polygons_calculation,
            theissen_polygons,
            get_csv,
            theissen_polygons_to_file,
            set_screen,
            set_layer_index,
            set_dataset_index,
            set_feature_index,
            create_new_dataset,
            add_field_to_schema,
            edit_dataset,
            add_feature_to_layer,
            set_epsg_srs_for_layer,
            select_tool_for_current_index,
            get_image_pixels,
            set_display,
            set_name_field,
            classify_current_raster,
            set_srs,
            reproject_layer,
        ])
        .path(s)
        .config(
            ExportConfig::new()
                .bigint(BigIntExportBehavior::Number)
                .formatter(prettier),
        )
        .build()
        .unwrap()
}

fn main() {
    let handlers = generate_handlers("../src/bindings.ts");
    let countries_path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("data/countries.geojson");
    eprintln!("Loading {:?}", &countries_path);
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
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppDataSync {
            data: Default::default(),
            default_data: PreloadedAppData { countries },
        })
        .invoke_handler(handlers)
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.open_devtools();
            let state = (*app.state::<AppDataSync>()).clone();
            let handle = app.handle();
            handle.manage(tauri::async_runtime::spawn(run_server(state)));
            let local_ip = local_ip().expect("Unable to retrieve local IP address");

            // Print the IP address and port
            let port = 80;
            println!("Server running at http://{}:{}/", local_ip, port);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct RasterSize {
    width: usize,
    length: usize,
    bands: usize,
}

#[tauri::command]
#[specta::specta]
fn get_band_sizes(state: AppState) -> Vec<RasterSize> {
    state.with_lock(|state| {
        state
            .shared
            .datasets
            .iter_mut()
            .map(|wrapped| {
                let dataset = &wrapped.dataset;
                let (width, length) = dataset.dataset.raster_size();
                let bands = dataset.dataset.raster_count();

                RasterSize {
                    width,
                    length,
                    bands,
                }
            })
            .collect()
    })
}

#[tauri::command]
#[specta::specta]
fn get_value_at_point(point: Point, state: AppState) -> Option<f64> {
    state
        .with_current_raster_band(|band| {
            let val = read_raster_data_enum_as(
                &band.band,
                (point.x.round() as isize, point.y.round() as isize),
                (1, 1),
                (1, 1),
                None,
            )?
            .to_f64()[0];
            if band.no_data_value().is_some_and(|ndv| val == ndv) {
                None
            } else {
                Some(val)
            }
        })
        .expect("Tried to get raster band and couldn't find it")
}

#[tauri::command]
#[specta::specta]
fn get_point_of_max_value(state: AppState) -> Option<Point> {
    state
        .with_current_raster_band(|band| {
            let data = read_raster_data(&band.band);
            let data_iter = data.indexed_iter();
            match band.no_data_value() {
                Some(no_data_value) => itertools::Either::Left(data_iter.filter(move |x| {
                    x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan()
                })),
                _ => itertools::Either::Right(data_iter.filter(|x| !x.1.is_nan())),
            }
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| Point::from_2d_index(index))
        })
        .unwrap()
}

#[tauri::command]
#[specta::specta]
fn get_point_of_min_value(state: AppState) -> Option<Point> {
    let mut guard = state.data.lock().unwrap();
    guard
        .with_current_raster_band(|band| {
            let data = read_raster_data(&band.band.band);
            let data_iter = data.indexed_iter();
            match band.band.no_data_value() {
                Some(no_data_value) => itertools::Either::Left(data_iter.filter(move |x| {
                    x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan()
                })),
                _ => itertools::Either::Right(data_iter.filter(|x| !x.1.is_nan())),
            }
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| Point::from_2d_index(index))
        })
        .unwrap()
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

/// Calculates the slope in degrees of a raster then calculates the tan of each value as we want actual slope values, not degrees
pub fn derivative_of_dataset(src: PathBuf, dest: PathBuf) -> std::io::Result<Output> {
    let tmp_path = "/tmp/tmp.tiff";
    Command::new("gdaldem")
        .arg("slope")
        .arg(src)
        .arg(tmp_path)
        .output();
    Command::new("gdal_calc.py")
        .arg("-A")
        .arg(tmp_path)
        .arg("--outfile")
        .arg(dest)
        .arg("--calc")
        .arg("'tan(A)'")
        .arg("--NoDataValue=-9999")
}

/// Calculates the slope in degrees of a raster then calculates the tan of each value as we want actual slope values, not degrees
pub fn aspect_of_dataset(src: PathBuf, dest: PathBuf) -> std::io::Result<Output> {
    let tmp_path = "/tmp/tmp.tiff";
    Command::new("gdaldem")
        .arg("aspect")
        .arg(src)
        .arg(tmp_path)
        .output();
    Command::new("gdal_calc.py")
        .arg("-A")
        .arg(tmp_path)
        .arg("--outfile")
        .arg(dest)
        .arg("--calc")
        .arg("'tan(A)'")
        .arg("--NoDataValue=-9999")
}
*/

pub trait IntoIndex {
    fn to_2d_index(self) -> (usize, usize);
}

impl IntoIndex for Point {
    fn to_2d_index(self) -> (usize, usize) {
        let Point { x, y } = self;
        (y as usize, x as usize)
    }
}

pub trait FromIndex {
    fn from_2d_index(index: (usize, usize)) -> Point;
}

impl FromIndex for Point {
    fn from_2d_index(index: (usize, usize)) -> Point {
        Self {
            x: index.1 as f64,
            y: index.0 as f64,
        }
    }
}

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

#[tauri::command]
#[specta::specta]
fn get_polygons_around_point(point: Point, layer: usize, state: AppState) -> Vec<PolygonInfo> {
    state
        .with_current_dataset_mut(|dataset, _| {
            let srs = dataset
                .dataset
                .dataset
                .spatial_ref()
                .unwrap()
                .to_wkt()
                .unwrap();
            let mut layer = dataset.dataset.get_vector(layer).unwrap();
            let features = layer.layer().features();
            features
                .flat_map(|feature| match feature.geometry() {
                    Some(geometry) => match geometry.to_geo().unwrap() {
                        geo_types::Geometry::Polygon(mut polygon) => {
                            if polygon.contains(&GeoPoint::from(point)) {
                                polygon
                                    .transform_crs_to_crs(&srs, "UTM 32N (EPSG:25832)")
                                    .unwrap();
                                let area = polygon.unsigned_area();
                                let fields = feature
                                    .fields()
                                    .map(|(name, value)| Field {
                                        name,
                                        value: value.into(),
                                    })
                                    .collect::<Vec<_>>();
                                Some(PolygonInfo { area, fields })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    None => None,
                })
                .collect()
        })
        .unwrap_or_else(Vec::new)
}

#[derive(Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct PolygonInfo {
    area: f64,
    fields: Vec<Field>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FeatureInfo {
    fields: Vec<Field>,
    geometry: Geometry,
}

impl FeatureInfo {
    fn new(geometry: Geometry, fields: Vec<Field>) -> Self {
        Self { geometry, fields }
    }
}

fn describe_polygon_internal(
    line: GeoLineString,
    countries: Vec<String>,
    towns: Vec<String>,
) -> ClosedLineDescription {
    let number_of_points = line.0.len();
    let perimeter = line.geodesic_length().round() / 1000.0;
    let mut points = line.clone().into_points();
    let distances = points
        .clone()
        .iter()
        .map_windows(|[a, b]| a.geodesic_distance(b))
        .mean();
    let (x, y) = {
        let envelope = line.envelope();
        let a = envelope.upper();
        let b = envelope.lower();
        let bearing = (-a.geodesic_bearing(b) + 90.0).to_radians();
        let distance = a.geodesic_distance(&b);
        let x = distance * bearing.cos().abs();
        let y = distance * bearing.sin().abs();
        (x, y)
    };
    if points.len() > 2 {
        points.push(points[1]);
    }
    let internal_angles = points
        .into_iter()
        .map_windows(|[start, mid, end]| start.geodesic_bearing(*mid) - mid.geodesic_bearing(*end));
    let waviness = internal_angles
        .clone()
        .filter(|x| !(*x == 0.0 || *x == -0.0))
        .map(|x| x.signum())
        .sum::<f64>();
    let area = line.geodesic_area_unsigned();
    ClosedLineDescription {
        x,
        y,
        towns,
        countries,
        perimeter,
        area,
        waviness,
        distances,
        number_of_points,
    }
}

fn describe_open_line(
    line: LineString,
    countries: Vec<String>,
    towns: Vec<String>,
) -> OpenLineDescription {
    let line = GeoLineString::from(line);
    let number_of_points = line.0.len();
    let length = line.geodesic_length().round() / 1000.0;
    let end_to_end_distance = GeoPoint::from(line.0[0])
        .geodesic_distance(&GeoPoint::from(*line.0.last().unwrap()))
        .round()
        / 1000.0;
    let points = line.clone().into_points();
    let distances = points
        .clone()
        .iter()
        .map_windows(|[a, b]| a.geodesic_distance(b))
        .mean();
    let (x, y) = {
        let a = points[0];
        let b = points.last().unwrap();
        let bearing = (-a.geodesic_bearing(*b) + 90.0).to_radians();
        let distance = a.geodesic_distance(b);
        let x = distance * bearing.cos();
        let y = distance * bearing.sin();
        (x, y)
    };
    let internal_angles = points
        .into_iter()
        .map_windows(|[start, mid, end]| start.geodesic_bearing(*mid) - mid.geodesic_bearing(*end));
    let waviness = internal_angles
        .clone()
        .filter(|x| !(*x == 0.0 || *x == -0.0))
        .map(|x| x.signum())
        .sum::<f64>();
    let total_angle = internal_angles.sum::<f64>();

    OpenLineDescription {
        x,
        y,
        towns,
        countries,
        angular_sum: total_angle,
        length,
        end_to_end_distance,
        waviness,
        distances,
        number_of_points,
    }
}

#[tauri::command]
#[specta::specta]
fn describe_line(
    line: LineString,
    srs: Option<String>,
    distance: f64,
    towns: usize,
    state: AppState,
) -> LineDescription {
    let line = GeoLineString::from(line);
    let line = match srs {
        Some(srs) => line.transformed_crs_to_crs(&srs, "WGS84").unwrap(),
        None => {
            eprint!("No srs provided for describing a line, assuming default srs of Wgs84");
            line
        }
    };
    let mut guard = state.data.lock().unwrap();
    let geometry = Geometry::LineString(line.clone().into());
    let geometry = SingleGeometry::try_from(geometry).unwrap();
    let countries = state.default_data.get_intersecting_countries(&geometry);
    let country_names = countries
        .iter()
        .flat_map(|x| x.get_field("ADMIN"))
        .collect();
    let towns = guard
        .get_towns_near_line(&line, countries.into_iter(), distance)
        .take(towns)
        .flat_map(|town| town.get_name())
        .collect();
    match line.is_closed() {
        true => LineDescription::Closed(describe_polygon_internal(line, country_names, towns)),
        false => LineDescription::Open(describe_open_line(line.into(), country_names, towns)),
    }
}

#[specta::specta]
fn _analyse_geom(line: LineString, mut guard: MutexGuard<'_, AppData>) {
    let line = GeoLineString::from(line);
    let mut contains = Vec::new();
    let mut contained_by = Vec::new();
    let mut crosses = Vec::new();
    for (dataset_idx, dataset) in guard.shared.datasets.iter_mut().enumerate() {
        for (layer_idx, mut layer) in &mut dataset.dataset.dataset.layers().enumerate() {
            for (feature_idx, feature) in layer.features().enumerate() {
                let geom = feature.geometry().expect("Feature has no geometry");
                let geom = geom
                    .to_geo()
                    .expect("Could not convert gdal geometry to geo geometry");
                //let line = Geometry::LineString { points: line }.into();
                let geom_type = geom.to_type();
                let info = (geom_type, dataset_idx, layer_idx, feature_idx);
                if geom.is_within(&line) {
                    contained_by.push(info);
                } else if geom.contains(&line) {
                    contains.push(info);
                } else if geom.intersects(&line) {
                    crosses.push(info);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type")]
pub enum LineDescription {
    Closed(ClosedLineDescription),
    Open(OpenLineDescription),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ClosedLineDescription {
    x: f64,
    y: f64,
    perimeter: f64,
    area: f64,
    countries: Vec<String>,
    towns: Vec<String>,
    waviness: f64,
    distances: f64,
    number_of_points: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct OpenLineDescription {
    x: f64,
    y: f64,
    length: f64,
    end_to_end_distance: f64,
    angular_sum: f64,
    countries: Vec<String>,
    towns: Vec<String>,
    waviness: f64,
    distances: f64,
    number_of_points: usize,
}

#[tauri::command]
#[specta::specta]
fn describe_polygon(polygon: Polygon, state: AppState) -> String {
    let polygon = GeoPolygon::from(polygon);
    let number_of_exteria_points = polygon.exterior().0.len();
    let area = polygon.chamberlain_duquette_unsigned_area() / 1000000.0;
    let perimeter = polygon.geodesic_perimeter() / 1000.0;
    let mut guard = state.data.lock().unwrap();
    let geometry = Geometry::Polygon(polygon.clone().into());
    let binding = geometry.try_into().unwrap();
    let countries = state.default_data.get_intersecting_countries(&binding);
    let country_names = countries
        .iter()
        .flat_map(|x| x.get_field("ADMIN"))
        .join(", ");
    let towns = guard
        .get_towns_in_polygon(&polygon, countries.into_iter())
        .take(20)
        .flat_map(|town| town.get_name())
        .join(", ");
    format!("A polygon with area {area}km and perimeter of {perimeter}km that has {number_of_exteria_points} exteria points, it intersects {country_names} and surrounds {towns}")
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, specta::Type)]
pub struct DistanceFromBoarder {
    name: String,
    distance: f64,
}

#[tauri::command]
#[specta::specta]
fn point_in_country(point: Point, state: AppState) -> Option<DistanceFromBoarder> {
    let mut guard = state.data.lock().unwrap();
    let point = guard.raster_point_to_wgs84(point.into());
    let country = state.default_data.find_country(&point)?;
    let closest_point = country.geom().exterior().closest_point(&point);
    let distance = match closest_point {
        Closest::Indeterminate => return None,
        Closest::SinglePoint(p) => p.geodesic_distance(&point).round(),
        Closest::Intersection(_) => 0.0,
    };
    Some(DistanceFromBoarder {
        name: country.get_field("ADMIN").unwrap(),
        distance,
    })
}

#[tauri::command]
#[specta::specta]
fn nearest_town(point: Point, state: AppState) -> Option<DistanceFromBoarder> {
    let mut guard = state.data.lock().unwrap();
    let point = guard.raster_point_to_wgs84(point.into());
    let country = state.default_data.find_country(&point)?;
    let code = country.get_code();
    let towns = guard.get_towns_by_code(code);
    towns
        .iter()
        .flat_map(|town| Some((town, town.nearest_point(&point)?)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(town, distance)| DistanceFromBoarder {
            name: town.get_name().unwrap(),
            distance: distance.round(),
        })
}

#[tauri::command]
#[specta::specta]
fn set_screen(screen: Screen, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.screen = screen;
}

#[tauri::command]
#[specta::specta]
fn set_dataset_index(index: usize, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.shared.datasets.set_index(index).unwrap();
}

#[tauri::command]
#[specta::specta]
fn set_layer_index(index: LayerIndex, state: AppState) {
    state
        .with_current_dataset_mut(|ds, _| {
            ds.layer_index = Some(index);
        })
        .expect("Tried to set layer index on nonexistant dataset");
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FeatureIndex {
    layer: LayerIndex,
    feature: usize,
}

#[tauri::command]
#[specta::specta]
fn set_feature_index(index: usize, state: AppState) -> Result<(), String> {
    state
        .with_current_vector_layer(|layer| layer.info.selected_feature = Some(index))
        .ok_or_else(|| "error".to_string())
}

#[tauri::command]
#[specta::specta]
fn create_new_dataset(driver_name: String, file: String, state: AppState) -> Result<(), String> {
    let mut guard = state.data.lock().unwrap();
    let mut dataset = WrappedDataset::new_vector(file, driver_name)?;
    dataset.add_layer()?;
    guard.shared.datasets.add(StatefulDataset {
        dataset,
        layer_index: Some(LayerIndex::Vector(0)),
        layer_info: vec![StatefulVectorInfo::default()],
        band_info: vec![],
    });
    guard.screen = Screen::Main;
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn add_field_to_schema(name: String, field_type: FieldType, state: AppState) -> Result<(), String> {
    state
        .with_current_vector_layer(move |layer| {
            layer
                .layer
                .layer()
                .create_defn_fields(&[(&name, field_type as u32)])
                .inspect_err(|e| eprintln!("{:?}", e))
                .map_err(|_| "Failed to add fields to schema".to_string())
        })
        .ok_or_else(|| "Tried to add field to schema when state is uninitialised".to_string())?
}

#[tauri::command]
#[specta::specta]
fn edit_dataset(state: AppState) -> Result<(), String> {
    eprintln!("here");
    state
        .with_current_dataset_mut(|dataset, _| {
            dataset.dataset.dataset = Dataset::open_ex(
                &dataset.dataset.file_name,
                DatasetOptions {
                    open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
                    ..Default::default()
                },
            )
            .inspect_err(|e| eprintln!("Err: {:?}", e))
            .map_err(|_| {
                format!(
                    "Could not open dataset {} in update mode",
                    &dataset.dataset.file_name
                )
            })?;
            dataset.dataset.editable = true;
            Ok(())
        })
        .expect("No dataset selected")
}

#[tauri::command]
#[specta::specta]
fn add_feature_to_layer(feature: FeatureInfo, state: AppState) -> Result<(), String> {
    let mut guard = state.data.lock().unwrap();
    guard
        .with_current_vector_layer(move |mut layer| {
            let geom = GeoGeometry::from(feature.geometry)
                .to_gdal()
                .map_err(|_| "Failed to convert geometry to gdal geometry")?;
            let fields = feature
                .fields
                .iter()
                .flat_map(|field| {
                    Some((
                        field.name.as_str(),
                        gdal::vector::FieldValue::from(field.value.clone()),
                    ))
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();

            layer
                .layer
                .layer()
                .create_feature_fields(geom, &fields.0, &fields.1)
                .inspect_err(|e| eprintln!("{:?}", e))
                .map_err(|_| "Failed to add fields to schema".to_string())?;
            eprintln!(
                "{:?}",
                FeatureInfo::from(layer.layer.layer().features().last().unwrap())
            );
            Ok(())
        })
        .ok_or_else(|| "Tried to add feature to layer when state is uninitialised".to_string())?
}

#[tauri::command]
#[specta::specta]
fn set_epsg_srs_for_layer(epsg_code: u32, state: AppState) -> Result<(), String> {
    let srs = SpatialRef::from_epsg(epsg_code)
        .map_err(|_| "Failed to get srs from epsg code".to_owned())?;
    let mut guard = state.data.lock().unwrap();
    guard
        .with_current_dataset_mut(move |dataset, _| {
            dataset
                .dataset
                .set_spatial_ref(&srs)
                .expect("Failed to set srs on dataset")
        })
        .ok_or_else(|| "Tried to add feature to layer when state is uninitialised".to_string())
}

#[tauri::command]
#[specta::specta]
fn select_tool_for_current_index(tool: ToolDataDiscriminants, state: AppState) {
    state.with_lock(|state| state.shared.tools_data.add_tool(tool))
}

#[tauri::command]
#[specta::specta]
fn get_image_pixels(state: AppState) -> Result<Vec<u8>, String> {
    state
        .with_current_raster_band(|band| {
            band.band()
                .read_band_as::<u8>()
                .expect("Not u8 data")
                .into_shape_and_vec()
                .1
        })
        .ok_or_else(|| "Couldn't read band data".to_owned())
}

#[tauri::command]
#[specta::specta]
fn set_display(state: AppState) {
    state
        .with_current_layer_mut(|mut layer| {
            layer.shared_mut().display = true;
        })
        .expect("Tried to edit nonexistant dataset");
}

#[tauri::command]
#[specta::specta]
fn set_name_field(field: String, state: AppState) {
    state
        .with_current_vector_layer(|layer| {
            layer.info.primary_field_name = Some(field);
        })
        .expect("Tried to edit nonexistant dataset");
}

#[tauri::command]
#[specta::specta]
fn classify_current_raster(dest: String, classifications: Vec<Classification>, state: AppState) {
    let classifications = classifications
        .into_iter()
        .map(Classification::to_calc_string)
        .join("+");
    state.with_current_dataset_mut(|dataset, _| {
        let mut cmd = Command::new("gdal_calc.py");
        cmd.arg("-A")
            .arg(&dataset.dataset.file_name)
            .arg(format!("--outfile={}", dest))
            .arg(format!("--calc=\"{}\"", classifications))
            .arg(format!(
                "--NoDataValue={}",
                dataset
                    .dataset
                    .dataset
                    .rasterband(1)
                    .unwrap()
                    .no_data_value()
                    .unwrap()
            ));
        let output = cmd.output().expect("Failed to classify raster");
        eprint!("{:?}", output);
    });
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct Classification {
    pub min: f64,
    pub max: f64,
    pub target: f64,
}

impl Classification {
    fn to_calc_string(self) -> String {
        format!("{}*(A>{})*(A<={})", self.target, self.min, self.max)
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Deserialize, Serialize, specta::Type,))]
#[serde(tag = "type", content = "value")]
pub enum Srs {
    Proj(String),
    Wkt(String),
    Esri(String),
    Epsg(u32),
}

fn list_projection_types() -> Vec<SrsDiscriminants> {
    SrsDiscriminants::iter().collect_vec()
}

#[tauri::command]
#[specta::specta]
fn set_srs(srs: Srs, state: AppState) {
    let srs = match srs {
        Srs::Proj(proj_string) => SpatialRef::from_proj4(&proj_string),
        Srs::Wkt(wkt_string) => SpatialRef::from_wkt(&wkt_string),
        Srs::Esri(esri_wkt) => SpatialRef::from_esri(&esri_wkt),
        Srs::Epsg(epsg_code) => SpatialRef::from_epsg(epsg_code),
    }
    .unwrap();
    state.with_current_dataset_mut(|ds, _| ds.dataset.dataset.set_spatial_ref(&srs).unwrap());
}

#[tauri::command]
#[specta::specta]
fn reproject_layer(srs: Srs, name: &str, state: AppState) {
    let srs = match srs {
        Srs::Proj(proj_string) => SpatialRef::from_proj4(&proj_string),
        Srs::Wkt(wkt_string) => SpatialRef::from_wkt(&wkt_string),
        Srs::Esri(esri_wkt) => SpatialRef::from_esri(&esri_wkt),
        Srs::Epsg(epsg_code) => SpatialRef::from_epsg(epsg_code),
    }
    .unwrap();
    // We've validated that the spatial reference system is valid
    let srs = srs.to_wkt().unwrap();
    state.with_current_dataset_mut(|ds, _| {
        let input_name = ds.dataset.file_name.clone();
        let layer = ds.get_current_layer();
        match layer {
            Some(StatefulLayerEnum::Vector(layer)) => {
                let mut command = Command::new("ogrtogr");
                command.arg("-t_srs").arg(&srs);
                command.arg(name).arg(&input_name);
                let output = command.output().unwrap();
                eprint!("{:?}", output)
            }
            Some(StatefulLayerEnum::Raster(band)) => {
                let mut command = Command::new("gdal_warp");
                let output = command.output().unwrap();
                eprint!("{:?}", output)
            }
            None => eprint!("No layer available"),
        }
    });
}

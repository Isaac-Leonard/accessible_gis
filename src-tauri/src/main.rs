// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(try_blocks)]
#![feature(iter_map_windows)]

mod files;
mod gdal_if;
mod geometry;
mod math;
mod state;
mod stats;
mod thiessen_polygons;

use files::get_csv;
use gdal::{vector::LayerAccess, Dataset};
use gdal_if::{read_raster_data, Envelope, Field, FieldValue, WrappedDataset};
use geo::{
    Area, ChamberlainDuquetteArea, Closest, ClosestPoint, Contains, GeodesicArea, GeodesicBearing,
    GeodesicDistance, GeodesicLength, Intersects, Point as GeoPoint, Within,
};
use geo_types::{LineString as GeoLineString, Polygon as GeoPolygon};
use geometry::{Geometry, LineString, Point, Polygon, SingleGeometry};
use itertools::Itertools;
use proj::Transform;
use rstar::{RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use specta::{
    collect_types,
    ts::{BigIntExportBehavior, ExportConfiguration},
};
use state::{AppData, AppState, FeatureNames, LayerOverview};
use statrs::statistics::Statistics;
use tauri_specta::ts;

use std::{cmp::Ordering, sync::MutexGuard};

use crate::{
    gdal_if::LocalFeatureInfo,
    geometry::ToGeometryType,
    state::{AppDataSync, Country, CountryImpl, PreloadedAppData},
    thiessen_polygons::{
        theissen_polygons, theissen_polygons_calculation, theissen_polygons_to_file,
    },
};

#[tauri::command]
#[specta::specta]
fn load_file(name: &str, state: AppState) -> String {
    let Ok(dataset) = Dataset::open(name) else {
        return "Something went wrong".to_owned();
    };
    let mut guard = state.data.lock().unwrap();
    guard.datasets.push(WrappedDataset {
        raster_data: try { read_raster_data(&dataset.rasterband(1).ok()?) },
        file_name: name.to_owned(),
        dataset,
    });
    "Success".to_owned() + &guard.datasets.len().to_string()
}

#[tauri::command]
#[specta::specta]
fn get_app_info(state: AppState) -> Vec<LayerDescriptor> {
    let guard = state.data.lock().unwrap();
    guard
        .datasets
        .iter()
        .enumerate()
        .flat_map(|(ref dataset_idx, dataset)| {
            let dataset_file = dataset.file_name.clone();
            dataset
                .dataset
                .layers()
                .enumerate()
                .map(|(layer_idx, layer)| LayerDescriptor {
                    kind: LayerInfo::Vector,
                    dataset_file: dataset_file.clone(),
                    dataset: *dataset_idx,
                    band: layer_idx,
                    srs: try { layer.spatial_ref()?.to_wkt().ok()? },
                    projection: dataset.dataset.projection(),
                })
                .chain((1..=dataset.dataset.raster_count()).map(|band_idx| {
                    let band = dataset.dataset.rasterband(band_idx).unwrap();
                    let (width, length) = band.size();
                    LayerDescriptor {
                        kind: LayerInfo::Raster { width, length },
                        dataset: *dataset_idx,
                        band: band_idx as usize,
                        dataset_file: dataset.file_name.clone(),
                        srs: try { dataset.dataset.spatial_ref().ok()?.to_wkt().ok()? },
                        projection: dataset.dataset.projection(),
                    }
                }))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn main() {
    ts::export_with_cfg(
        collect_types![
            load_file,
            get_app_info,
            get_band_sizes,
            get_value_at_point,
            get_point_of_max_value,
            get_point_of_min_value,
            get_polygons_around_point,
            get_layer_info,
            get_feature_info,
            get_feature_names,
            describe_line,
            describe_polygon,
            point_in_country,
            nearest_town,
            theissen_polygons_calculation,
            theissen_polygons,
            get_csv,
            theissen_polygons_to_file,
        ]
        .unwrap(),
        ExportConfiguration::default().bigint(BigIntExportBehavior::Number),
        "../src/bindings.ts",
    )
    .unwrap();
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
        .manage(AppDataSync {
            data: Default::default(),
            default_data: PreloadedAppData { countries },
        })
        .invoke_handler(tauri::generate_handler![
            load_file,
            get_app_info,
            get_band_sizes,
            get_value_at_point,
            get_point_of_max_value,
            get_point_of_min_value,
            get_polygons_around_point,
            get_layer_info,
            get_feature_info,
            get_feature_names,
            describe_line,
            describe_polygon,
            point_in_country,
            nearest_town,
            theissen_polygons_calculation,
            theissen_polygons,
            get_csv,
            theissen_polygons_to_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, specta::Type)]
pub struct RasterSize {
    width: usize,
    length: usize,
    bands: usize,
}

#[tauri::command]
#[specta::specta]
fn get_band_sizes(state: AppState) -> Vec<RasterSize> {
    let guard = state.data.lock().unwrap();
    guard
        .datasets
        .iter()
        .map(|wrapped| {
            let dataset = &wrapped.dataset;
            let (width, length) = dataset.raster_size();
            let bands = dataset.raster_count() as usize;

            RasterSize {
                width,
                length,
                bands,
            }
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
fn get_value_at_point(point: Point, layer: LayerDescriptor, state: AppState) -> ValueType {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[layer.dataset];
    match layer.kind {
        LayerInfo::Raster { .. } => {
            let val = dataset.raster_data.as_ref().unwrap()[point.to_2d_index()];

            let band = dataset.dataset.rasterband(1).unwrap();
            if band
                .no_data_value()
                .is_some_and(|no_data_value| val == no_data_value)
            {
                ValueType::Err("NoData".to_owned())
            } else {
                ValueType::Value(val)
            }
        }
        LayerInfo::Vector => ValueType::Err("Not implemented for Vector data yet".to_owned()),
    }
}

#[tauri::command]
#[specta::specta]
fn get_point_of_max_value(layer: LayerDescriptor, state: AppState) -> Point {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[layer.dataset];
    let data_iter = dataset.raster_data.as_ref().unwrap().indexed_iter();

    if let Some(no_data_value) = dataset
        .dataset
        .rasterband(layer.band as isize)
        .unwrap()
        .no_data_value()
    {
        data_iter
            .filter(|x| x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan())
            .max_by(|a, b| a.1.total_cmp(b.1))
    } else {
        data_iter
            .filter(|x| !x.1.is_nan())
            .max_by(|a, b| a.1.total_cmp(b.1))
    }
    .map(|(index, _)| Point::from_2d_index(index))
    .unwrap()
}

#[tauri::command]
#[specta::specta]
fn get_point_of_min_value(layer: LayerDescriptor, state: AppState) -> Point {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[0];
    let data_iter = dataset.raster_data.as_ref().unwrap().indexed_iter();

    match dataset
        .dataset
        .rasterband(layer.band as isize)
        .unwrap()
        .no_data_value()
    {
        Some(no_data_value) => data_iter
            .filter(|x| x.1.total_cmp(&no_data_value) != Ordering::Equal && !x.1.is_nan())
            .min_by(|a, b| a.1.total_cmp(b.1)),
        _ => data_iter
            .filter(|x| !x.1.is_nan())
            .max_by(|a, b| a.1.total_cmp(b.1)),
    }
    .map(|(index, _)| Point::from_2d_index(index))
    .unwrap()
}

#[derive(Serialize, specta::Type)]
#[serde(untagged)]
enum ValueType {
    Value(f64),
    Err(String),
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
    #[serde(flatten)]
    kind: LayerInfo,
    dataset: usize,
    band: usize,
    dataset_file: String,
    srs: Option<String>,
    projection: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, specta::Type)]
#[serde(tag = "type")]
pub enum LayerInfo {
    Raster { width: usize, length: usize },
    Vector,
}

#[tauri::command]
#[specta::specta]
fn get_polygons_around_point(
    point: Point,
    layer: LayerDescriptor,
    state: AppState,
) -> Vec<PolygonInfo> {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[layer.dataset];
    let srs = dataset.dataset.spatial_ref().unwrap().to_wkt().unwrap();
    let mut layer = dataset.dataset.layer(layer.band as isize).unwrap();
    let features = layer.features();
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
}

#[tauri::command]
#[specta::specta]
fn get_feature_info(
    feature: usize,
    layer: LayerDescriptor,
    state: AppState,
) -> Option<FeatureInfo> {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[layer.dataset];
    let layer = dataset.dataset.layer(layer.band as isize).unwrap();
    let feature = layer.feature(feature as u64)?;

    let fields = feature
        .fields()
        .map(|(name, value)| Field {
            name,
            value: value.into(),
        })
        .collect::<Vec<_>>();
    Some(FeatureInfo {
        fields,
        geometry: feature.geometry().unwrap().to_geo().unwrap().into(),
    })
}

#[derive(Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct PolygonInfo {
    area: f64,
    fields: Vec<Field>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct FeatureInfo {
    fields: Vec<Field>,
    geometry: Geometry,
}

#[tauri::command]
#[specta::specta]
fn get_layer_info(layer: LayerDescriptor, state: AppState) -> LayerOverview {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[layer.dataset];
    let mut layer = dataset.dataset.layer(layer.band as isize).unwrap();
    let extent: Option<Envelope> = layer.get_extent().ok().map(Into::into);
    LayerOverview {
        name: layer.name(),
        extent,
        features: layer.feature_count() as usize,
        field_names: layer
            .features()
            .flat_map(|feature| {
                feature
                    .fields()
                    .map(|field| field.0)
                    .collect::<Vec<String>>()
            })
            .unique()
            .collect(),
    }
}

#[tauri::command]
#[specta::specta]
fn get_feature_names(
    name_field: String,
    layer: LayerDescriptor,
    state: AppState,
) -> Option<FeatureNames> {
    let guard = state.data.lock().unwrap();
    let dataset = &guard.datasets[layer.dataset];
    let mut layer = dataset.dataset.layer(layer.band as isize).unwrap();
    Some(FeatureNames {
        features: layer
            .features()
            .map(|feature| Some(FieldValue::from(feature.field(&name_field).ok()?).to_string()))
            .collect::<Option<_>>()?,
        field: name_field,
    })
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
    for (dataset_idx, dataset) in guard.datasets.iter_mut().enumerate() {
        for (layer_idx, mut layer) in &mut dataset.dataset.layers().enumerate() {
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
fn point_in_country(
    layer: LayerDescriptor,
    point: Point,
    state: AppState,
) -> Option<DistanceFromBoarder> {
    let guard = state.data.lock().unwrap();
    let point = guard.point_to_wgs84(point.into(), layer).unwrap();
    eprintln!("Point: {:?}", point);
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
fn nearest_town(
    layer: LayerDescriptor,
    point: Point,
    state: AppState,
) -> Option<DistanceFromBoarder> {
    let mut guard = state.data.lock().unwrap();
    let point = guard.point_to_wgs84(point.into(), layer).unwrap();
    eprintln!("Point: {:?}", point);
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

use std::sync::MutexGuard;

use gdal::vector::LayerAccess;
use geo::{
    Area, ChamberlainDuquetteArea, Closest, ClosestPoint, Contains, GeodesicArea, GeodesicBearing,
    GeodesicDistance, GeodesicLength, Intersects, Within,
};
use geo_types::{LineString as GeoLineString, Point as GeoPoint, Polygon as GeoPolygon};
use itertools::Itertools;
use proj::Transform;
use rstar::RTreeObject;
use serde::{Deserialize, Serialize};
use statrs::statistics::Statistics;

use crate::{
    dataset_collection::NonEmptyDelegatorImpl,
    gdal_if::Field,
    geometry::{Geometry, LineString, Point, Polygon, SingleGeometry, ToGeometryType},
    state::{AppData, AppState, CountryImpl},
};

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
pub fn describe_line(
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
            for feature in layer.features() {
                let fid = feature.fid().unwrap();
                let geom = feature.geometry().expect("Feature has no geometry");
                let geom = geom
                    .to_geo()
                    .expect("Could not convert gdal geometry to geo geometry");
                //let line = Geometry::LineString { points: line }.into();
                let geom_type = geom.to_type();
                let info = (geom_type, dataset_idx, layer_idx, fid);
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
pub fn describe_polygon(polygon: Polygon, state: AppState) -> String {
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
pub fn point_in_country(point: Point, state: AppState) -> Option<DistanceFromBoarder> {
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
pub fn get_polygons_around_point(point: Point, layer: usize, state: AppState) -> Vec<PolygonInfo> {
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

#[tauri::command]
#[specta::specta]
pub fn nearest_town(point: Point, state: AppState) -> Option<DistanceFromBoarder> {
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

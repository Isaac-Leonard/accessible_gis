use tauri::Manager;

use geo::{
    ChamberlainDuquetteArea, Closest, ClosestPoint, GeodesicArea, GeodesicBearing,
    GeodesicDistance, GeodesicLength,
};
use geo_types::{LineString as GeoLineString, Point as GeoPoint, Polygon as GeoPolygon};
use itertools::Itertools;
use proj::Transform;
use rstar::RTreeObject;
use serde::{Deserialize, Serialize};
use statrs::statistics::Statistics;
use tauri::AppHandle;

use crate::{
    gdal_if::Field,
    geometry::{Geometry, LineString, Point, Polygon, SingleGeometry},
    state::{AppState, CountryImpl},
};

#[tauri::command]
#[specta::specta]
pub fn describe_polygon(polygon: Polygon, state: AppState, app: AppHandle) -> String {
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
        .get_towns_in_polygon(&polygon, countries.into_iter(), app.path())
        .take(20)
        .flat_map(|town| town.get_name())
        .join(", ");
    format!(
        "A polygon with area {area}km and perimeter of {perimeter}km that has {number_of_exteria_points} exteria points, it intersects {country_names} and surrounds {towns}"
    )
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

#[derive(Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct PolygonInfo {
    area: f64,
    fields: Vec<Field>,
}

#[tauri::command]
#[specta::specta]
pub fn nearest_town(point: Point, state: AppState, app: AppHandle) -> Option<DistanceFromBoarder> {
    let mut guard = state.data.lock().unwrap();
    let point = guard.raster_point_to_wgs84(point.into());
    let country = state.default_data.find_country(&point)?;
    let code = country.get_code();
    let towns = guard.get_towns_by_code(code, app.path());
    towns
        .iter()
        .flat_map(|town| Some((town, town.nearest_point(&point)?)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(town, distance)| DistanceFromBoarder {
            name: town.get_name().unwrap(),
            distance: distance.round(),
        })
}

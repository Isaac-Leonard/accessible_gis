use std::path::PathBuf;

use csv::Reader;
use gdal::vector::{LayerAccess, ToGdal};
use geo::Area;
use geo_types::{
    MultiPoint as GeoMultiPoint, MultiPolygon as GeoMultiPolygon, Point as GeoPoint,
    Polygon as GeoPolygon,
};
use itertools::Itertools;
use proj::Transform;
use serde::{Deserialize, Serialize};

use crate::{
    dataset_collection::{StatefulDataset, StatefulVectorInfo},
    gdal_if::{get_driver_for_file, WrappedDataset},
    geometry::{MultiPoint, Point, Polygon},
    state::AppState,
};

#[tauri::command]
#[specta::specta]
pub fn theissen_polygons(points: MultiPoint, srs: String) -> Vec<Polygon> {
    let points = GeoMultiPoint::from(points);
    let points = points
        .0
        .into_iter()
        .map(|point| point.transformed_crs_to_crs("Wgs84", &srs).unwrap())
        .collect::<Vec<_>>();
    geos::compute_voronoi(&points, None, 0.0, false)
        .unwrap()
        .into_iter()
        .map_into()
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn theissen_polygons_calculation(records: Vec<ThiessenPolygonRecord>, srs: String) -> Vec<f64> {
    let points = records
        .iter()
        .map(|r| {
            GeoPoint::from(r.point)
                .transformed_crs_to_crs("Wgs84", &srs)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let polygons = geos::compute_voronoi(&points, None, 0.0, false).unwrap();
    let values = records.iter().map(|data| {
        let mut reader = Reader::from_path(&data.file).expect("Could not read file");
        reader
            .records()
            .skip(data.start_line)
            .map(|r| {
                let record = r.expect("Invalid record");
                let field = record
                    .get(data.column)
                    .unwrap_or_else(|| panic!("Field missing for column {}", data.column));
                field
                    .parse::<f64>()
                    .expect("Could not parse field as number")
            })
            .collect::<Vec<_>>()
    });
    let areas = polygons.iter().map(GeoPolygon::unsigned_area);
    let total_areas = areas.clone().sum::<f64>();
    let x = areas
        .zip(values)
        .map(|(area, rain)| rain.iter().map(|v| v * area).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut transposed = vec![vec![0.0; x.len()]; x[0].len()];
    for (poly_i, col) in x.iter().enumerate() {
        for (time_i, val) in col.iter().enumerate() {
            transposed[time_i][poly_i] = *val
        }
    }
    transposed
        .into_iter()
        .map(|row| row.into_iter().sum::<f64>() / total_areas)
        .collect()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, specta::Type)]
pub struct ThiessenPolygonRecord {
    point: Point,
    file: String,
    start_line: usize,
    column: usize,
}

#[tauri::command]
#[specta::specta]
pub fn theissen_polygons_to_file(points: MultiPoint, srs: String, file: PathBuf, state: AppState) {
    let points = GeoMultiPoint::from(points);
    let points = points.transformed_crs_to_crs("Wgs84", &srs).unwrap();
    let polygons = geos::compute_voronoi(&points.0, None, 0.0, false).unwrap();
    let polygons = GeoMultiPolygon::new(polygons)
        .transformed_crs_to_crs("Wgs84", &srs)
        .unwrap();
    let driver = get_driver_for_file(&file)
        .unwrap_or_else(|| panic!("Could not find driver for file {}", file.display()));

    let mut dataset = driver.create_vector_only(&file).unwrap();
    let mut layer = dataset.create_layer(Default::default()).unwrap();
    layer.create_feature(polygons.to_gdal().unwrap()).unwrap();
    dataset.flush_cache().unwrap();
    let mut guard = state.data.lock().unwrap();
    let wrapped_dataset = WrappedDataset {
        file_name: file.to_string_lossy().to_string(),
        dataset,
        editable: true,
    };
    guard.shared.datasets.add(StatefulDataset {
        dataset: wrapped_dataset,
        layer_index: None,
        layer_info: vec![StatefulVectorInfo::default()],
        band_info: vec![],
    });
}

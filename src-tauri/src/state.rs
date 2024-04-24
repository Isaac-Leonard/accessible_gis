use std::{collections::HashMap, sync::Mutex, vec::IntoIter};

use gdal::{vector::LayerAccess, Dataset, GeoTransformEx};
use geo::{
    Closest, ClosestPoint, Contains, GeodesicDistance, Intersects, LineString as GeoLineString,
};
use geo_types::{LineString, Point, Polygon};
use itertools::Itertools;
use proj::{Coord, Transform};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    gdal_if::{Field, LocalFeatureInfo, WrappedDataset},
    geometry::{AsPoint, GeoGeometry, GeoSingleGeometry, SingleGeometry},
    Envelope, LayerDescriptor, LayerInfo,
};

pub type AppState<'a> = State<'a, AppDataSync>;

pub struct AppDataSync {
    pub data: Mutex<AppData>,
    pub default_data: PreloadedAppData,
}

#[derive(Default)]
pub struct AppData {
    pub datasets: Vec<WrappedDataset>,
    pub towns: HashMap<String, Vec<LocalFeatureInfo>>,
}

impl AppData {
    pub fn get_towns_by_code(&mut self, code: String) -> &Vec<LocalFeatureInfo> {
        self.towns.entry(code).or_insert_with_key(|code| {
            let dataset_path = std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .join(format!("data/countries/{code}.geojson"));
            let towns_dataset = Dataset::open(dataset_path).unwrap();
            let mut layer = towns_dataset.layer(0).unwrap();
            layer.features().map_into::<LocalFeatureInfo>().collect()
        })
    }

    pub fn point_to_wgs84(&self, point: Point, layer: LayerDescriptor) -> Option<Point> {
        let dataset = self.get_raw_dataset(layer.clone())?;
        let point = match layer.kind {
            LayerInfo::Vector => point,
            LayerInfo::Raster { .. } => {
                let transform = dataset
                    .geo_transform()
                    .expect("Raster datasets should have a geo transform");
                let point = transform.apply(point.x(), point.y());
                Point::from_xy(point.0, point.1)
            }
        };
        Some(point.transformed_crs_to_crs(&layer.srs?, "WGS84").unwrap())
    }

    pub fn get_towns_in_polygon<'a>(
        &mut self,
        polygon: &Polygon,
        countries: impl Iterator<Item = &'a Country>,
    ) -> IntoIter<LocalFeatureInfo> {
        countries
            .flat_map(move |country| {
                let polygon2 = polygon.clone();
                self.get_towns_by_code(country.get_code())
                    .clone()
                    .into_iter()
                    .filter(move |town| polygon2.contains(&town.geometry))
                    .collect::<Vec<_>>()
            })
            .sorted_by(|a, b| {
                // Sort in ascending order
                str::parse::<i64>(&b.get_field("population").unwrap())
                    .unwrap()
                    .cmp(&str::parse::<i64>(&a.get_field("population").unwrap()).unwrap())
            })
    }

    pub fn get_towns_near_line<'a>(
        &mut self,
        polygon: &LineString,
        countries: impl Iterator<Item = &'a Country>,
        distance: f64,
    ) -> IntoIter<LocalFeatureInfo> {
        countries
            .flat_map(move |country| {
                let polygon2 = polygon.clone();
                self.get_towns_by_code(country.get_code())
                    .clone()
                    .into_iter()
                    .filter(move |town| {
                        match GeoLineString::from(polygon2.clone())
                            .closest_point(town.geometry.as_point().unwrap())
                        {
                            Closest::Indeterminate => false,
                            Closest::Intersection(_) => true,
                            Closest::SinglePoint(p) => {
                                p.geodesic_distance(town.geometry.as_point().unwrap()) < distance
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .sorted_by(|a, b| {
                // Sort in ascending order
                str::parse::<i64>(&b.get_field("population").unwrap())
                    .unwrap()
                    .cmp(&str::parse::<i64>(&a.get_field("population").unwrap()).unwrap())
            })
    }

    pub fn get_raw_dataset(&self, layer: LayerDescriptor) -> Option<&Dataset> {
        self.datasets.get(layer.dataset).map(|x| &x.dataset)
    }

    pub fn get_raw_dataset_mut(&mut self, layer: LayerDescriptor) -> Option<&Dataset> {
        self.datasets.get_mut(layer.dataset).map(|x| &x.dataset)
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct LayerOverview {
    pub name: String,
    pub extent: Option<Envelope>,
    pub features: usize,
    pub field_names: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FeatureNames {
    pub field: String,
    pub features: Vec<String>,
}

pub struct PreloadedAppData {
    pub countries: RTree<Country>,
}

impl PreloadedAppData {
    /// Point must be a WGS84 point
    pub fn find_country(&self, point: &Point) -> Option<&Country> {
        self.countries
            .iter()
            .find(|country| country.geom().contains(point))
    }

    pub fn get_intersecting_countries<'a>(
        &'a self,
        geometry: &'a SingleGeometry,
    ) -> Vec<&'a Country> {
        let from = GeoSingleGeometry::from(geometry.clone());
        self.countries
            .locate_in_envelope_intersecting(&from.envelope())
            // TODO: Not sure if this line is needed
            .filter(|x| SingleGeometry::Polygon(x.geom().clone().into()).intersects(geometry))
            .collect()
    }
}

pub type Country = GeomWithData<Polygon, Vec<Field>>;
impl From<Country> for LocalFeatureInfo {
    fn from(value: Country) -> Self {
        Self {
            geometry: GeoGeometry::Polygon(value.geom().clone()),
            fields: value.data,
        }
    }
}

impl From<LocalFeatureInfo> for Vec<Country> {
    fn from(value: LocalFeatureInfo) -> Self {
        let polygons = match value.geometry {
            GeoGeometry::Polygon(polygon) => vec![polygon],
            GeoGeometry::MultiPolygon(polygons) => polygons.0,
            _ => panic!("Unexpected geometry in country"),
        };
        polygons
            .into_iter()
            .map(|p| Country::new(p, value.fields.clone()))
            .collect()
    }
}

pub trait CountryImpl {
    fn get_code(&self) -> String;
    fn get_field(&self, name: &str) -> Option<String>;
}

impl CountryImpl for Country {
    fn get_code(&self) -> String {
        self.data
            .iter()
            .find(|x| x.name == "ISO_A2")
            .unwrap()
            .value
            .to_string()
    }

    fn get_field(&self, name: &str) -> Option<String> {
        Some(
            self.data
                .iter()
                .find(|f| f.name == name)?
                .value
                .to_string()
                .clone(),
        )
    }
}

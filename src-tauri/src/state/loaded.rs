use std::{collections::HashMap, vec::IntoIter};

use gdal::{vector::LayerAccess, Dataset};
use geo::{Closest, ClosestPoint, Contains, GeodesicDistance};
use geo_types::{LineString, Point, Polygon};
use itertools::Itertools;
use tauri::{path::PathResolver, Runtime};

use crate::{
    dataset_collection::DatasetCollection,
    gdal_if::{LocalFeatureInfo, WrappedDataset},
    geometry::AsPoint,
};

use super::{
    gis::{dataset::StatefulDataset, raster::StatefulRasterBand, vector::StatefulVectorLayer},
    preloaded::Country,
    settings::GlobalSettings,
    user_state::UserState,
    CountryImpl, Screen,
};

const DEFAULT_SETTINGS_FILE_NAME: &str = "settings.json";

pub struct AppData {
    pub towns: HashMap<String, Vec<LocalFeatureInfo>>,
    pub screen: Screen,
    pub shared: UserState,
    pub errors: Vec<String>,
    settings: GlobalSettings,
}

impl AppData {
    pub fn open_dataset(&mut self, name: String) -> Result<&mut StatefulDataset, String> {
        self.shared.datasets.open(name, &self.settings)
    }

    pub fn new<R: Runtime>(resolver: &PathResolver<R>) -> Self {
        Self {
            towns: HashMap::new(),
            screen: Screen::Main,
            shared: UserState::default(),
            errors: Vec::new(),
            settings: GlobalSettings::read(resolver),
        }
    }

    pub fn create_from_current_dataset<E, F>(
        &mut self,
        f: F,
    ) -> Option<Result<&mut StatefulDataset, E>>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, E>,
    {
        self.shared.create_from_current_dataset(f, &self.settings)
    }

    pub fn with_current_vector_layer<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(StatefulVectorLayer) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| {
            dataset.get_current_layer()?.try_into_vector().ok().map(f)
        })?
    }

    pub fn with_current_dataset_mut<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        self.shared.with_current_dataset_mut(f)
    }

    pub fn raster_point_to_wgs84(&mut self, point: Point) -> Point {
        self.with_current_raster_band(|band| band.band.point_to_wgs84(point))
            .flatten()
            .expect("Expected raster band and couldn't find it")
    }

    pub fn with_current_raster_band<T, F>(&mut self, f: F) -> Option<T>
    where
        F: Fn(StatefulRasterBand) -> T,
    {
        self.shared
            .datasets
            .with_current_dataset_mut(|dataset, _| {
                let band = dataset.get_current_layer()?.try_into_raster().ok()?;
                let res = f(band);
                dataset
                    .dataset
                    .save_changes()
                    .expect("Could not flush cache");
                Some(res)
            })
            .flatten()
    }

    pub fn datasets(&self) -> &DatasetCollection {
        &self.shared.datasets
    }

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
                        match polygon2
                            .clone()
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

    pub fn settings(&self) -> &GlobalSettings {
        &self.settings
    }

    pub fn set_settings<R: Runtime>(
        &mut self,
        settings: GlobalSettings,
        resolver: &PathResolver<R>,
    ) -> &GlobalSettings {
        self.settings = settings;
        self.settings.write_to_file(resolver);
        &self.settings
    }
}

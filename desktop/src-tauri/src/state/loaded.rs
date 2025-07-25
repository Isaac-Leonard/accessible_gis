use std::{collections::HashMap, path::Path, vec::IntoIter};

use gdal::{Dataset, vector::LayerAccess};
use geo::{Closest, ClosestPoint, Contains, GeodesicDistance};
use geo_types::{LineString, Point, Polygon};
use itertools::Itertools;
use serde::Serialize;
use tauri::{Runtime, Wry, path::PathResolver};
use uuid::Uuid;

use crate::{
    errors::{ApplicationError, ErrorDetails},
    gdal_if::{LocalFeatureInfo, WrappedDataset},
    geometry::AsPoint,
};

use super::{
    CountryImpl, Screen,
    dataset_collection::{NonEmptyDelegator, NonEmptyDelegatorImpl},
    gis::dataset::StatefulDataset,
    preloaded::Country,
    projects::Project,
    settings::GlobalSettings,
};

pub struct AppData {
    pub towns: HashMap<String, Vec<LocalFeatureInfo>>,
    pub project: Option<Project>,
    pub screen: Screen,
    pub errors: ErrorList,
    settings: GlobalSettings,
}

impl AppData {
    pub fn open_dataset(&mut self, name: impl AsRef<Path>) -> Option<&mut StatefulDataset> {
        self.with_project_fallible(|project| {
            project
                .datasets
                .open(name, &project.settings)
                .map_err(|err| ErrorDetails::OpenDatasetError(err.clone()))
        })
    }

    pub fn new<R: Runtime>(resolver: &PathResolver<R>) -> Self {
        Self {
            towns: HashMap::new(),
            screen: Screen::Main,
            project: None,
            errors: ErrorList::new(),
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
        self.with_project(|project| {
            project.create_from_current_dataset(f, &project.settings.clone())
        })?
    }

    pub fn with_project<'a, T, F>(&'a mut self, f: F) -> Option<T>
    where
        F: FnOnce(&'a mut Project) -> T,
    {
        self.project.as_mut().map(f)
    }

    pub fn with_project_fallible<'a, T, F>(&'a mut self, f: F) -> Option<T>
    where
        F: FnOnce(&'a mut Project) -> Result<T, ErrorDetails>,
    {
        match self.project.as_mut().map(f) {
            Some(Ok(v)) => Some(v),
            Some(Err(e)) => {
                self.errors.push(e.into());
                None
            }
            None => None,
        }
    }

    pub fn raster_point_to_wgs84(&mut self, point: Point) -> Point {
        self.with_current_raster_band(|band| band.band.point_to_wgs84(point))
            .flatten()
            .expect("Expected raster band and couldn't find it")
    }

    pub fn get_towns_by_code(
        &mut self,
        code: String,
        resolver: &PathResolver<Wry>,
    ) -> &Vec<LocalFeatureInfo> {
        self.towns.entry(code).or_insert_with_key(|code| {
            let dataset_path = resolver
                .resolve(
                    format!("data/countries/{code}.geojson"),
                    tauri::path::BaseDirectory::Resource,
                )
                .unwrap();
            let towns_dataset = Dataset::open(dataset_path).unwrap();
            let mut layer = towns_dataset.layer(0).unwrap();
            layer.features().map_into::<LocalFeatureInfo>().collect()
        })
    }

    pub fn get_towns_in_polygon<'a>(
        &mut self,
        polygon: &Polygon,
        countries: impl Iterator<Item = &'a Country>,
        resolver: &PathResolver<Wry>,
    ) -> IntoIter<LocalFeatureInfo> {
        countries
            .flat_map(move |country| {
                let polygon2 = polygon.clone();
                self.get_towns_by_code(country.get_code(), resolver)
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
        resolver: &PathResolver<Wry>,
    ) -> IntoIter<LocalFeatureInfo> {
        countries
            .flat_map(move |country| {
                let polygon2 = polygon.clone();
                self.get_towns_by_code(country.get_code(), resolver)
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

impl NonEmptyDelegator for AppData {
    fn get_non_empty(&self) -> Option<&super::dataset_collection::NonEmptyDatasetCollection> {
        self.project.as_ref()?.get_non_empty()
    }

    fn get_non_empty_mut(
        &mut self,
    ) -> Option<&mut super::dataset_collection::NonEmptyDatasetCollection> {
        self.project.as_mut()?.get_non_empty_mut()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct ErrorList(Vec<ApplicationError>);

impl ErrorList {
    pub fn to_vec(&self) -> Vec<ApplicationError> {
        self.0.clone()
    }

    pub fn push(&mut self, err: ApplicationError) {
        self.0.push(err)
    }

    fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&mut self, id: Uuid) -> Option<&mut ApplicationError> {
        return self.0.iter_mut().find(|err| err.id == id);
    }
}

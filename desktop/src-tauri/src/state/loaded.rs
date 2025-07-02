use std::{collections::HashMap, path::Path, vec::IntoIter};

use gdal::{Dataset, vector::LayerAccess};
use geo::{Closest, ClosestPoint, Contains, GeodesicDistance};
use geo_types::{LineString, Point, Polygon};
use itertools::Itertools;
use serde::Serialize;
use tauri::{Runtime, Wry, path::PathResolver};
use uuid::Uuid;

use crate::{
    dataset_collection::DatasetCollection,
    errors::{ApplicationError, ErrorDetails},
    gdal_if::{LocalFeatureInfo, OpenDatasetError, WrappedDataset},
    geometry::AsPoint,
    web_socket::{GisMessage, RasterMessage, VectorMessage},
};

use super::{
    CountryImpl, Screen,
    gis::{dataset::StatefulDataset, raster::StatefulRasterBand, vector::StatefulVectorLayer},
    preloaded::Country,
    settings::GlobalSettings,
    user_state::UserState,
};

pub struct AppData {
    pub towns: HashMap<String, Vec<LocalFeatureInfo>>,
    pub screen: Screen,
    pub shared: UserState,
    pub errors: ErrorList,
    settings: GlobalSettings,
    pub prefered_display_fields: Vec<String>,
}

impl AppData {
    /// Gets all of the data needed to update the touch devices configuration
    /// Note that names of enums and structs are still not finalised as the end result is not yet clear
    pub fn get_touch_device_settings(&mut self) -> Option<GisMessage> {
        let settings = &self.shared.get_raster_to_display()?.info.audio_settings;
        Some(GisMessage {
            raster: RasterMessage {
                min_freq: settings.min_freq,
                max_freq: settings.max_freq,
            },
            vector: VectorMessage {
                prefered_keys: self.prefered_display_fields.clone(),
            },
        })
    }
    pub fn open_dataset(
        &mut self,
        name: impl AsRef<Path>,
    ) -> Result<&mut StatefulDataset, OpenDatasetError> {
        match self.shared.datasets.open(name, &self.settings) {
            Ok(dataset) => Ok(dataset),
            Err(err) => {
                self.errors
                    .push(ErrorDetails::OpenDatasetError(err.clone()).into());
                Err(err)
            }
        }
    }

    pub fn new<R: Runtime>(resolver: &PathResolver<R>) -> Self {
        Self {
            towns: HashMap::new(),
            screen: Screen::Main,
            shared: UserState::default(),
            errors: ErrorList::new(),
            settings: GlobalSettings::read(resolver),
            prefered_display_fields: Vec::new(),
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

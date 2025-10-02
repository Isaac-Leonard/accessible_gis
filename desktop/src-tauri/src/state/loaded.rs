use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime, path::PathResolver};
use uuid::Uuid;

use crate::{
    errors::{ApplicationError, ErrorDetails},
    gdal_if::WrappedDataset,
};

use super::{
    Screen,
    dataset_collection::{NonEmptyDelegator, NonEmptyDelegatorImplExt},
    gis::{
        combined::StatefulLayerEnum, dataset::StatefulDataset, raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
    projects::Project,
    settings::GlobalSettings,
};

pub struct AppData {
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

    pub fn open_dataset_multi(&mut self, name: impl AsRef<Path>) -> Option<()> {
        self.with_project_fallible(|project| {
            project
                .datasets
                .open_multi(name, &project.settings)
                .map_err(|err| ErrorDetails::OpenDatasetError(err.clone()))
        })
    }

    pub fn new(app: &AppHandle) -> Self {
        let mut errors = ErrorList::new();
        Self {
            screen: Screen::Main,
            project: None,
            settings: GlobalSettings::read(app.path())
                .map_err(|err| errors.push(err.into()))
                .unwrap_or_default(),
            errors,
        }
    }

    pub fn create_from_current_dataset<F>(
        &mut self,
        f: F,
    ) -> Option<Result<&mut StatefulDataset, ErrorDetails>>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, ErrorDetails>,
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

    pub fn settings(&self) -> &GlobalSettings {
        &self.settings
    }

    pub fn set_settings<R: Runtime>(
        &mut self,
        settings: GlobalSettings,
        resolver: &PathResolver<R>,
    ) -> &GlobalSettings {
        self.settings = settings;
        if let Err(err) = self.settings.write_to_file(resolver) {
            self.errors.push(err.into())
        };
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

/// Methods to run a function and catch errors
impl AppData {
    fn with_fallible<T, E, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut AppData) -> Option<Result<T, E>>,
        E: Into<ErrorDetails>,
    {
        match f(self) {
            Some(Ok(val)) => Some(val),
            Some(Err(err)) => {
                let err = err.into();
                self.errors.push(err.into());
                None
            }
            None => None,
        }
    }

    pub fn with_current_dataset_mut_fallible<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> Result<T, ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_dataset_mut(f))
    }

    pub fn with_current_layer_mut_fallible<T, E, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(StatefulLayerEnum) -> Result<T, E>,
        E: Into<ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_layer_mut(f))
    }

    pub fn with_current_raster_band_fallible<T, E, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulRasterBand) -> Result<T, E>,
        E: Into<ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_raster_band(f))
    }

    pub fn with_current_vector_layer_fallible<T, E, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulVectorLayer) -> Result<T, E>,
        E: Into<ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_vector_layer(f))
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
        self.0.iter_mut().find(|err| err.id == id)
    }
}

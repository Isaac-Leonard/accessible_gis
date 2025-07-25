pub mod dataset_collection;
pub mod gis;
mod loaded;
mod preloaded;
pub mod projects;
pub mod settings;
mod ui_state;

use std::sync::{Arc, Mutex};

use dataset_collection::NonEmptyDelegatorImplExt;
use gis::{raster::StatefulRasterBand, vector::StatefulVectorLayer};
use projects::Project;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{errors::ErrorDetails, gdal_if::Envelope};

pub use loaded::*;
pub use preloaded::*;

use self::gis::{combined::StatefulLayerEnum, dataset::StatefulDataset};

pub type AppState<'a> = State<'a, AppDataSync>;

#[derive(Clone)]
pub struct AppDataSync {
    pub data: Arc<Mutex<AppData>>,
    pub default_data: PreloadedAppData,
}

impl AppDataSync {
    pub fn with_lock<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut AppData) -> T,
    {
        // If this panics then something has gone wrong elsewhere.
        let mut guard = self.data.lock().unwrap();
        f(&mut guard)
    }

    pub fn with_project<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Project) -> T,
    {
        self.with_lock(|state| state.with_project(f))
    }

    pub fn with_project_fallible<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Project) -> Result<T, ErrorDetails>,
    {
        self.with_lock(|state| state.with_project_fallible(f))
    }
}

// We can't implement NonEmptyDelegator or NonEmptyDelegatorImpl on this because they both require returning inner data and NonEmptyDelegatorImplExt requires taking &mut self, which won't work for the same reasons
impl AppDataSync {
    pub fn with_current_dataset_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        self.with_lock(|state| state.with_current_dataset_mut(f))
    }

    pub fn with_current_layer_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(StatefulLayerEnum) -> T,
    {
        self.with_lock(|state| state.with_current_layer_mut(f))
    }

    pub fn with_current_raster_band<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulRasterBand) -> T,
    {
        self.with_lock(|state| state.with_current_raster_band(f))
    }

    pub fn with_current_vector_layer<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulVectorLayer) -> T,
    {
        self.with_lock(|state| state.with_current_vector_layer(f))
    }
}

/// These methods take a function that may return errors, run it and add the error to the apps error list.
impl AppDataSync {
    pub fn with_fallible<T, E, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut AppData) -> Option<Result<T, E>>,
        E: Into<ErrorDetails>,
    {
        self.with_lock(|state| match f(state) {
            Some(Ok(val)) => Some(val),
            Some(Err(err)) => {
                let err = err.into();
                state.errors.push(err.into());
                None
            }
            None => None,
        })
    }

    pub fn with_current_dataset_mut_fallible<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> Result<T, ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_dataset_mut(f))
    }

    pub fn with_current_layer_mut_fallible<T, E, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(StatefulLayerEnum) -> Result<T, E>,
        E: Into<ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_layer_mut(f))
    }

    pub fn with_current_raster_band_fallible<T, E, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulRasterBand) -> Result<T, E>,
        E: Into<ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_raster_band(f))
    }

    pub fn with_current_vector_layer_fallible<T, E, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulVectorLayer) -> Result<T, E>,
        E: Into<ErrorDetails>,
    {
        self.with_fallible(|state| state.with_current_vector_layer(f))
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

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type, Default)]
pub enum Screen {
    #[default]
    Main,
    NewDataset,
    Settings,
    TouchDevice,
    Errors,
}

pub mod dataset_collection;
pub mod gis;
mod loaded;
mod preloaded;
pub mod projects;
pub mod settings;
mod ui_state;

use std::sync::{Arc, Mutex};

use projects::Project;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{errors::ErrorDetails, gdal_if::Envelope};

pub use loaded::*;
pub use preloaded::*;

use self::gis::{
    combined::StatefulLayerEnum, dataset::StatefulDataset, raster::StatefulRasterBand,
    vector::StatefulVectorLayer,
};

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

    pub fn with_current_layer_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(StatefulLayerEnum) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| Some(f(dataset.get_current_layer()?)))
            .flatten()
            .flatten()
    }

    pub fn with_current_raster_band<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulRasterBand) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| {
            let index = *dataset.layer_index?.as_raster()?;
            let mut band = dataset.get_raster(index)?;
            Some(f(&mut band))
        })
        .flatten()
        .flatten()
    }

    pub fn with_current_vector_layer<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulVectorLayer) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| {
            let index = *dataset.layer_index?.as_vector()?;
            let mut layer = dataset.get_vector(index)?;
            Some(f(&mut layer))
        })
        .flatten()
        .flatten()
    }

    pub fn with_current_dataset_mut<T, F>(&self, f: F) -> Option<Option<T>>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        self.with_lock(|state| state.with_current_dataset_mut(f))
    }

    pub fn with_current_dataset_mut_fallible<T, F>(&self, f: F) -> Option<Option<T>>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> Result<T, ErrorDetails>,
    {
        self.with_lock(|state| match state.with_current_dataset_mut(f) {
            Some(Some(Ok(v))) => Some(Some(v)),
            Some(Some(Err(err))) => {
                state.errors.push(err.into());
                None
            }
            None | Some(None) => None,
        })
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

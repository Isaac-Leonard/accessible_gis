mod loaded;
mod preloaded;
mod user_state;

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    dataset_collection::{StatefulDataset, StatefulLayerEnum, StatefulVectorLayer},
    gdal_if::WrappedRasterBand,
    Envelope,
};

pub use loaded::*;
pub use preloaded::*;
pub use user_state::*;

pub type AppState<'a> = State<'a, AppDataSync>;

#[derive(Clone)]
pub struct AppDataSync {
    pub data: Arc<Mutex<AppData>>,
    pub default_data: PreloadedAppData,
}

impl AppDataSync {
    pub fn with_lock<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut UserState) -> T,
    {
        // If this panics then something has gone wrong elsewhere.
        let mut guard = self.data.lock().unwrap();
        f(&mut guard.shared)
    }

    pub fn with_current_layer_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(StatefulLayerEnum) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| Some(f(dataset.get_current_layer()?)))
            .flatten()
    }

    pub fn with_current_raster_band<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut WrappedRasterBand) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| {
            let index = *dataset.layer_index?.as_raster()?;
            let mut band = dataset.dataset.get_raster(index)?;
            Some(f(&mut band))
        })
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
    }

    pub fn with_current_dataset_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        // If this panics then something has gone wrong elsewhere.
        let mut guard = self.data.lock().unwrap();
        guard.with_current_dataset_mut(f)
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
    ThiessenPolygons,
    NewDataset,
}

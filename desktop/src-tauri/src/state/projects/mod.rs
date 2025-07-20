use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ApplicationError, ErrorDetails},
    gdal_if::{Srs, WrappedDataset},
    web_socket::{GisMessage, RasterMessage, VectorMessage},
};

use super::{
    dataset_collection::{DatasetCollection, NonEmptyDelegatorImpl},
    gis::{
        combined::RasterIndex, dataset::StatefulDataset, raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
    settings::GlobalSettings,
};

pub struct Project {
    location: PathBuf,
    srs: Srs,
    pub datasets: DatasetCollection,
    pub settings: GlobalSettings,
    pub prefered_display_fields: Vec<String>,
    pub raster_to_display: Option<RasterIndex>,
}

impl Project {
    pub fn new(path: PathBuf, settings: &GlobalSettings) -> Result<Self, ApplicationError> {
        let project = Self {
            location: path,
            srs: Srs::default(),
            settings: settings.clone(),
            datasets: DatasetCollection::Empty,
            raster_to_display: None,
            prefered_display_fields: vec![],
        };
        project.save()?;
        Ok(project)
    }

    pub fn load(path: PathBuf) -> Result<Self, ApplicationError> {
        let serialised_project =
            std::fs::read_to_string(&path).map_err(|err| ErrorDetails::IoError(err.to_string()))?;

        let stored_project = serde_json::from_str::<StoredProject>(&serialised_project)
            .map_err(|err| ErrorDetails::SerdeError(err.to_string()))?;
        Self::from_stored_project(stored_project, path)
    }

    fn save(&self) -> Result<(), ApplicationError> {
        let stored_project = self.to_stored_project();
        let serialised_project = serde_json::to_string_pretty(&stored_project)
            .map_err(|err| ErrorDetails::SerdeError(err.to_string()))?;

        std::fs::write(&self.location, serialised_project)
            .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
        Ok(())
    }

    fn to_stored_project(&self) -> StoredProject {
        StoredProject {
            srs: self.srs.clone(),
            datasets: self
                .datasets
                .iter()
                .map(|ds| ds.dataset.file_name.clone())
                .collect(),
            settings: self.settings.clone(),
            prefered_display_fields: self.prefered_display_fields.clone(),
            raster_to_display: self.raster_to_display.clone(),
        }
    }

    fn from_stored_project(
        project: StoredProject,
        path: PathBuf,
    ) -> Result<Self, ApplicationError> {
        let mut datasets = DatasetCollection::Empty;
        for dataset in project.datasets {
            datasets
                .open(dataset, &project.settings)
                .map_err(ErrorDetails::OpenDatasetError)?;
        }

        Ok(Self {
            location: path,
            srs: project.srs,
            datasets,
            settings: project.settings.clone(),
            prefered_display_fields: project.prefered_display_fields.clone(),
            raster_to_display: project.raster_to_display,
        })
    }

    pub fn display_current_raster(&mut self, raster: Option<RasterIndex>) {
        self.raster_to_display = raster;
    }

    pub fn get_raster_to_display(&mut self) -> Option<StatefulRasterBand> {
        self.datasets.get_raster(self.raster_to_display?)
    }

    pub fn get_raster_index_to_display(&mut self) -> Option<RasterIndex> {
        self.raster_to_display
    }

    pub fn get_vectors_for_display(&mut self) -> Vec<StatefulVectorLayer> {
        self.datasets
            .get_vectors()
            .filter(|layer| layer.info.display)
            .collect_vec()
    }

    pub fn create_from_current_dataset<E, F>(
        &mut self,
        f: F,
        settings: &GlobalSettings,
    ) -> Option<Result<&mut StatefulDataset, E>>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, E>,
    {
        self.datasets.create_from_current_dataset(f, settings)
    }

    pub fn with_current_dataset_mut<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        self.datasets.with_current_dataset_mut(f)
    }

    /// Gets all of the data needed to update the touch devices configuration
    /// Note that names of enums and structs are still not finalised as the end result is not yet clear
    pub fn get_touch_device_settings(&mut self) -> GisMessage {
        let settings = match self.get_raster_to_display() {
            Some(band) => &band.info.audio_settings,
            None => self.settings.get_default_audio(),
        };
        GisMessage {
            raster: RasterMessage {
                min_freq: settings.min_freq,
                max_freq: settings.max_freq,
            },
            vector: VectorMessage {
                prefered_keys: self.prefered_display_fields.clone(),
            },
        }
    }

    pub fn with_current_raster_band<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulRasterBand) -> T,
    {
        self.with_current_dataset_mut(|dataset, _| {
            let index = *dataset.layer_index?.as_raster()?;
            let mut band = dataset.get_raster(index)?;
            Some(f(&mut band))
        })
        .flatten()
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct StoredProject {
    srs: Srs,
    pub datasets: Vec<PathBuf>,
    pub settings: GlobalSettings,
    pub prefered_display_fields: Vec<String>,
    pub raster_to_display: Option<RasterIndex>,
}

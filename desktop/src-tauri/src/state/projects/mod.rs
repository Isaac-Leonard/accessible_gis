use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ApplicationError, ErrorDetails},
    gdal_if::{LayerIndex, OpenDatasetError, Srs, WrappedDataset},
    web_socket::{GisMessage, RasterMessage, VectorMessage},
};

use super::{
    dataset_collection::{
        DatasetCollection, NonEmptyDelegator, NonEmptyDelegatorImpl, NonEmptyDelegatorImplExt,
    },
    gis::{
        combined::RasterIndex,
        dataset::StatefulDataset,
        raster::{StatefulRasterBand, StatefulRasterInfo},
        vector::{StatefulVectorInfo, StatefulVectorLayer},
    },
    settings::GlobalSettings,
    tools::{SavedToolOutputAction, Tool, UserDefinedTool, get_built_in_tools},
};

pub struct Project {
    location: PathBuf,
    pub srs: Srs,
    pub datasets: DatasetCollection,
    pub settings: GlobalSettings,
    pub prefered_display_fields: Vec<String>,
    pub raster_to_display: Option<RasterIndex>,
    pub use_labels: bool,
    pub announce_leaving: bool,
    pub announce_geometry_type: bool,
    pub tools: Vec<Box<dyn Tool>>,
    pub tool_outputs: Vec<SavedToolOutputAction>,
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
            use_labels: false,
            announce_geometry_type: true,
            announce_leaving: true,
            // TODO: Have a list of project tools and global tools
            tools: get_built_in_tools(),
            tool_outputs: Vec::new(),
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

    pub fn save(&self) -> Result<(), ErrorDetails> {
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
                .map(|ds| StoredDataset::WithInfo {
                    path: ds.dataset.file_name.clone(),
                    layer_index: ds.layer_index.clone(),
                    vector_info: ds.vector_info.clone(),
                    raster_info: ds.raster_info.clone(),
                })
                .collect(),
            settings: self.settings.clone(),
            prefered_display_fields: self.prefered_display_fields.clone(),
            raster_to_display: self.raster_to_display,
            use_labels: self.use_labels,
            announce_leaving: self.announce_leaving,
            announce_geometry_type: self.announce_geometry_type,
            tools: self
                .tools
                .iter()
                .filter_map(|tool| tool.as_user_defined_tool())
                .collect(),
            tool_outputs: self.tool_outputs.clone(),
        }
    }

    fn from_stored_project(
        project: StoredProject,
        path: PathBuf,
    ) -> Result<Self, ApplicationError> {
        let mut datasets = DatasetCollection::Empty;
        for dataset in project.datasets {
            load_dataset(&mut datasets, dataset, &project.settings)
                .map_err(ErrorDetails::OpenDatasetError)?;
        }

        let mut tools = get_built_in_tools();
        tools.extend(
            project
                .tools
                .into_iter()
                .map(|tool| Box::new(tool) as Box<dyn Tool>),
        );

        Ok(Self {
            location: path,
            srs: project.srs,
            datasets,
            settings: project.settings.clone(),
            prefered_display_fields: project.prefered_display_fields.clone(),
            raster_to_display: project.raster_to_display,
            use_labels: project.use_labels,
            announce_leaving: project.announce_leaving,
            announce_geometry_type: project.announce_geometry_type,
            tools,
            tool_outputs: project.tool_outputs,
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

    pub fn create_from_current_dataset<F>(
        &mut self,
        f: F,
        settings: &GlobalSettings,
    ) -> Option<Result<&mut StatefulDataset, ErrorDetails>>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, ErrorDetails>,
    {
        self.datasets.create_from_current_dataset(f, settings)
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
                use_labels: self.use_labels,
                announce_leaving: self.announce_leaving,
                announce_geometry_type: self.announce_geometry_type,
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

    pub fn get_tools(&self) -> Vec<Box<dyn Tool>> {
        self.tools.clone()
    }
}

impl NonEmptyDelegator for Project {
    fn get_non_empty(&self) -> Option<&super::dataset_collection::NonEmptyDatasetCollection> {
        self.datasets.get_non_empty()
    }

    fn get_non_empty_mut(
        &mut self,
    ) -> Option<&mut super::dataset_collection::NonEmptyDatasetCollection> {
        self.datasets.get_non_empty_mut()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredProject {
    pub srs: Srs,
    pub datasets: Vec<StoredDataset>,
    pub settings: GlobalSettings,
    pub prefered_display_fields: Vec<String>,
    pub raster_to_display: Option<RasterIndex>,
    #[serde(default)]
    pub use_labels: bool,
    #[serde(default = "get_true")]
    pub announce_leaving: bool,
    #[serde(default = "get_true")]
    pub announce_geometry_type: bool,
    #[serde(default)]
    tools: Vec<UserDefinedTool>,
    #[serde(default)]
    pub tool_outputs: Vec<SavedToolOutputAction>,
}

/// This just exists to use true as a default value for serde
fn get_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StoredDataset {
    RawFile(PathBuf),
    WithInfo {
        path: PathBuf,
        layer_index: Option<LayerIndex>,
        vector_info: Vec<StatefulVectorInfo>,
        raster_info: Vec<StatefulRasterInfo>,
    },
}

fn load_dataset(
    datasets: &mut DatasetCollection,
    dataset: StoredDataset,
    settings: &GlobalSettings,
) -> Result<(), OpenDatasetError> {
    match dataset {
        StoredDataset::RawFile(path) => {
            datasets.open(path, &settings)?;
        }
        StoredDataset::WithInfo {
            path,
            layer_index,
            vector_info,
            raster_info,
        } => {
            let dataset = WrappedDataset::open(path)?;
            datasets.add(StatefulDataset {
                dataset,
                layer_index,
                vector_info,
                raster_info,
            });
        }
    }
    Ok(())
}

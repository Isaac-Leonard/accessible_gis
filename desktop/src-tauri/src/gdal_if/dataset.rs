use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use gdal::{
    Dataset, DatasetOptions, DriverManager, GdalOpenFlags, Metadata, errors::GdalError,
    spatial_ref::SpatialRef, vector::Layer,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumIter};

use crate::{commands::EditDatasetError, dataset_collection::IndexedLayer, errors::ErrorDetails};

use super::{LayerEnum, WrappedLayer, errors::MyGdalError, raster::WrappedRasterBand};

#[derive(Debug)]
pub struct WrappedDataset {
    pub file_name: PathBuf,
    dataset: Dataset,
    pub editable: bool,
}

impl WrappedDataset {
    pub fn new(file_name: PathBuf, dataset: Dataset) -> Self {
        Self {
            file_name,
            dataset,
            editable: false,
        }
    }

    pub fn reopen_as_editable(&mut self) -> Result<(), ErrorDetails> {
        let open_writable_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
            ..Default::default()
        };

        let editable_dataset =
            Dataset::open_ex(&self.file_name, open_writable_options).map_err(|err| {
                EditDatasetError::OpenError(OpenDatasetError {
                    name: self.file_name.clone(),
                    gdal_error: err.into(),
                })
            })?;
        self.dataset = editable_dataset;
        self.editable = true;
        Ok(())
    }

    pub fn dataset(&self) -> &Dataset {
        &self.dataset
    }

    pub fn dataset_mut(&mut self) -> &mut Dataset {
        &mut self.dataset
    }

    /// Gets all the metadata for the dataset and returns it indexed by domain and subindexed by metadata key
    pub fn get_metadata(&self) -> DatasetMetadata {
        let metadata = self
            .dataset
            .metadata_domains()
            .into_iter()
            .filter_map(|domain| {
                Some((
                    domain.clone(),
                    self.dataset
                        .metadata_domain(&domain)?
                        .iter()
                        .filter_map(|metadata| metadata.split_once("="))
                        .map(|(a, b)| (a.to_string(), b.to_string()))
                        .collect::<BTreeMap<String, String>>(),
                ))
            })
            .collect();
        DatasetMetadata(metadata)
    }

    pub fn get_all_layers(&self) -> Result<Vec<IndexedLayer>, ErrorDetails> {
        let mut layers = Vec::new();
        for (index, layer) in self.dataset.layers().enumerate() {
            layers.push(IndexedLayer {
                layer: LayerEnum::Layer(WrappedLayer::new(layer, index)),
                layer_index: index,
            });
        }
        let srs = self.dataset.spatial_ref().ok();
        let geo_transform = self.dataset.geo_transform().ok();
        self.dataset
            .rasterbands()
            .map_ok(|band| WrappedRasterBand::new(band, geo_transform, srs.clone()))
            // We need to return early if theres an error getting a band
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| ErrorDetails::Other(err.to_string()))?
            .into_iter()
            .enumerate()
            .for_each(|(index, band)| {
                layers.push(IndexedLayer {
                    layer: LayerEnum::Band(band),
                    layer_index: index + 1,
                })
            });
        Ok(layers)
    }

    pub fn vectors(&mut self) -> Vec<WrappedLayer> {
        self.dataset
            .layers()
            .enumerate()
            .map(|(index, layer)| WrappedLayer::new(layer, index))
            .collect_vec()
    }

    pub fn bands(&mut self) -> Result<Vec<WrappedRasterBand>, ErrorDetails> {
        let srs = self.dataset.spatial_ref().ok();
        let geo_transform = self.dataset.geo_transform().ok();
        self.dataset
            .rasterbands()
            .map_ok(|band| WrappedRasterBand::new(band, geo_transform, srs.clone()))
            .try_collect()
            .map_err(|err| ErrorDetails::Other(err.to_string()))
    }

    pub fn save_changes(&mut self) -> Result<(), FlushCacheError> {
        self.dataset.flush_cache().map_err(FlushCacheError::new)
    }

    pub fn open(name: impl AsRef<Path>) -> Result<Self, OpenDatasetError> {
        Ok(WrappedDataset {
            file_name: name.as_ref().to_path_buf(),
            dataset: Dataset::open(&name).map_err(|err| OpenDatasetError {
                name: name.as_ref().to_path_buf(),
                gdal_error: err.into(),
            })?,
            editable: false,
        })
    }

    pub fn open_multi(name: impl AsRef<Path>) -> Result<Vec<Self>, OpenDatasetError> {
        let base_dataset = Dataset::open(&name).map_err(|err| OpenDatasetError {
            name: name.as_ref().to_path_buf(),
            gdal_error: err.into(),
        })?;
        get_all_subdatasets(&base_dataset)
    }

    /// Sometimes we need to manually open or create a dataset and need a way to wrap it
    /// We assume for now that it is not editable
    pub fn wrap_existing(dataset: Dataset, file_name: impl AsRef<Path>) -> Self {
        Self {
            file_name: file_name.as_ref().to_path_buf(),
            dataset,
            editable: false,
        }
    }

    pub fn new_vector(
        name: impl AsRef<Path>,
        driver: String,
    ) -> Result<Self, DatasetCreationError> {
        let driver = DriverManager::get_driver_by_name(&driver).map_err(|err| {
            DatasetCreationError::DriverError(MissingDriverError {
                driver,
                gdal_error: err.into(),
            })
        })?;
        let mut dataset = driver.create_vector_only(&name).map_err(|err| {
            DatasetCreationError::CreationError(CreationError {
                file: name.as_ref().to_path_buf(),
                driver: driver.long_name(),
                gdal_error: err.into(),
            })
        })?;
        dataset.flush_cache().map_err(|err| {
            DatasetCreationError::FlushCacheError(FlushCacheError {
                gdal_error: err.into(),
            })
        })?;
        Ok(WrappedDataset {
            file_name: name.as_ref().to_path_buf(),
            dataset,
            editable: true,
        })
    }

    pub fn add_layer(&mut self) -> Result<Layer, String> {
        if !self.editable {
            return Err("Dataset is not editable".to_string());
        }
        let layer = self
            .dataset
            .create_layer(Default::default())
            .map_err(|_| "Failed to create layer".to_string())?;
        Ok(layer)
    }

    pub fn get_vector(&mut self, index: usize) -> Option<WrappedLayer> {
        let layer = self.dataset.layer(index).ok()?;
        Some(WrappedLayer::new(layer, index))
    }

    pub fn get_raster(&mut self, index: usize) -> Option<WrappedRasterBand> {
        let band = self.dataset.rasterband(index).ok()?;
        Some(WrappedRasterBand::new(
            band,
            self.dataset.geo_transform().ok(),
            self.dataset.spatial_ref().ok(),
        ))
    }

    pub fn spatial_ref(&self) -> Result<SpatialRef, GdalError> {
        self.dataset.spatial_ref()
    }

    pub fn set_spatial_ref(
        &mut self,
        spatial_ref: &gdal::spatial_ref::SpatialRef,
    ) -> gdal::errors::Result<()> {
        self.dataset.set_spatial_ref(spatial_ref)
    }

    pub fn save(&mut self) -> Result<(), ErrorDetails> {
        self.dataset.flush_cache().map_err(|err| {
            ErrorDetails::EditDatasetError(EditDatasetError::SaveError(FlushCacheError {
                gdal_error: err.into(),
            }))
        })
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Deserialize, Serialize, specta::Type,))]
#[serde(tag = "type", content = "value")]
pub enum Srs {
    Proj(String),
    Wkt(String),
    Esri(String),
    Epsg(u32),
}

impl Default for Srs {
    fn default() -> Self {
        Self::Epsg(4326)
    }
}

impl Srs {
    pub fn try_to_gdal(self) -> Result<SpatialRef, GdalError> {
        match self {
            Srs::Proj(proj_string) => SpatialRef::from_proj4(&proj_string),
            Srs::Wkt(wkt_string) => SpatialRef::from_wkt(&wkt_string),
            Srs::Esri(esri_wkt) => SpatialRef::from_esri(&esri_wkt),
            Srs::Epsg(epsg_code) => SpatialRef::from_epsg(epsg_code),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub enum DatasetCreationError {
    CreationError(CreationError),
    DriverError(MissingDriverError),
    FlushCacheError(FlushCacheError),
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct CreationError {
    pub file: PathBuf,
    pub driver: String,
    pub gdal_error: MyGdalError,
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct MissingDriverError {
    pub driver: String,
    pub gdal_error: MyGdalError,
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct FlushCacheError {
    gdal_error: MyGdalError,
}

impl FlushCacheError {
    pub fn new(gdal_error: GdalError) -> Self {
        Self {
            gdal_error: gdal_error.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, specta::Type)]
pub struct OpenDatasetError {
    pub name: PathBuf,
    pub gdal_error: MyGdalError,
}

fn get_all_subdatasets(ds: &Dataset) -> Result<Vec<WrappedDataset>, OpenDatasetError> {
    // Fetch all metadata items in the SUBDATASETS domain
    let Some(subdatasets_metadata) = ds.metadata_domain("SUBDATASETS") else {
        return Ok(Vec::new());
    };

    subdatasets_metadata
        .iter()
        .filter_map(|metadata| {
            let (key, value) = metadata.split_once("=")?;
            let key = key.trim();
            let value = value.trim();
            if key.contains("_NAME") {
                Some(value)
            } else {
                None
            }
        })
        .map(WrappedDataset::open)
        .try_collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct DatasetMetadata(pub BTreeMap<String, BTreeMap<String, String>>);

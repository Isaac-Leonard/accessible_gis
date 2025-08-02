use std::path::{Path, PathBuf};

use gdal::{Dataset, DriverManager, errors::GdalError, spatial_ref::SpatialRef, vector::Layer};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumIter};

use crate::{commands::EditDatasetError, dataset_collection::IndexedLayer, errors::ErrorDetails};

use super::{LayerEnum, WrappedLayer, errors::MyGdalError, raster::WrappedRasterBand};

#[derive(Debug)]
pub struct WrappedDataset {
    pub file_name: PathBuf,
    pub dataset: Dataset,
    pub editable: bool,
}

impl WrappedDataset {
    pub fn get_all_layers(&self) -> Result<Vec<IndexedLayer>, ErrorDetails> {
        let mut layers = Vec::new();
        for (index, layer) in self.dataset.layers().enumerate() {
            layers.push(IndexedLayer {
                layer: LayerEnum::Layer(WrappedLayer { layer, index }),
                layer_index: index,
            });
        }
        let srs = self.dataset.spatial_ref().ok();
        let geo_transform = self.dataset.geo_transform().ok();
        self.dataset
            .rasterbands()
            .map(|band| {
                Ok(WrappedRasterBand {
                    geo_transform,
                    srs: srs.clone(),
                    band: band.map_err(|err| ErrorDetails::Other(err.to_string()))?,
                })
            })
            // We need to return early if theres an error getting a band
            .collect::<Result<Vec<_>, _>>()?
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
            .map(|(index, layer)| WrappedLayer { layer, index })
            .collect_vec()
    }

    pub fn bands(&mut self) -> Result<Vec<WrappedRasterBand>, ErrorDetails> {
        let srs = self.dataset.spatial_ref().ok();
        let geo_transform = self.dataset.geo_transform().ok();
        self.dataset
            .rasterbands()
            .map(|band| {
                Ok(WrappedRasterBand {
                    geo_transform,
                    srs: srs.clone(),
                    band: band.map_err(|err| ErrorDetails::Other(err.to_string()))?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
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
        Some(WrappedLayer { layer, index })
    }

    pub fn get_raster(&mut self, index: usize) -> Option<WrappedRasterBand> {
        let band = self.dataset.rasterband(index).ok()?;
        Some(WrappedRasterBand {
            band,
            geo_transform: self.dataset.geo_transform().ok(),
            srs: self.dataset.spatial_ref().ok(),
        })
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

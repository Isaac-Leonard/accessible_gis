use std::{ffi::c_int, path::Path};

use gdal::{errors::GdalError, spatial_ref::SpatialRef, vector::Layer, Dataset, DriverManager};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumIter};

use crate::dataset_collection::IndexedLayer;

use super::{raster::WrappedRasterBand, LayerEnum, WrappedLayer};

pub struct WrappedDataset {
    pub file_name: String,
    pub dataset: Dataset,
    pub editable: bool,
}

impl WrappedDataset {
    pub fn get_all_layers(&self) -> Vec<IndexedLayer> {
        let mut layers = Vec::new();
        for index in 0..self.dataset.layer_count() {
            let c_layer =
                unsafe { gdal_sys::OGR_DS_GetLayer(self.dataset.c_dataset(), index as c_int) };
            if !c_layer.is_null() {
                let layer = unsafe { Layer::from_c_layer(&self.dataset, c_layer) };
                layers.push(IndexedLayer {
                    layer: LayerEnum::Layer(WrappedLayer { layer, index }),
                    layer_index: index,
                });
            } else {
                eprint!("Retrieving layer gave null pointer")
            }
        }
        let srs = self.dataset.spatial_ref().and_then(|srs| srs.to_wkt()).ok();
        let geo_transform = self.dataset.geo_transform().ok();
        self.dataset
            .rasterbands()
            .map(|band| WrappedRasterBand {
                geo_transform,
                srs: srs.clone(),
                band: band.unwrap(),
            })
            .enumerate()
            .for_each(|(index, band)| {
                layers.push(IndexedLayer {
                    layer: LayerEnum::Band(band),
                    layer_index: index + 1,
                })
            });
        layers
    }

    pub fn vectors(&mut self) -> Vec<WrappedLayer> {
        self.dataset
            .layers()
            .enumerate()
            .map(|(index, layer)| WrappedLayer { layer, index })
            .collect_vec()
    }

    pub fn bands(&mut self) -> Vec<WrappedRasterBand> {
        let srs = self.dataset.spatial_ref().and_then(|srs| srs.to_wkt()).ok();
        let geo_transform = self.dataset.geo_transform().ok();
        self.dataset
            .rasterbands()
            .map(|band| WrappedRasterBand {
                geo_transform,
                srs: srs.clone(),
                band: band.unwrap(),
            })
            .collect_vec()
    }

    pub fn save_changes(&mut self) -> gdal::errors::Result<()> {
        self.dataset.flush_cache()
    }

    pub fn open(name: impl AsRef<Path>) -> Result<Self, String> {
        let dataset = Dataset::open(&name).map_err(|_| "Something went wrong".to_owned())?;
        Ok(WrappedDataset {
            file_name: name.as_ref().to_str().unwrap().to_owned(),
            dataset,
            editable: false,
        })
    }

    /// Sometimes we need to manually open or create a dataset and need a way to wrap it
    /// We assume for now that it is not editable
    pub fn wrap_existing(dataset: Dataset, file_name: String) -> Self {
        Self {
            file_name,
            dataset,
            editable: false,
        }
    }

    pub fn new_vector(name: String, driver: String) -> Result<Self, String> {
        let driver = DriverManager::get_driver_by_name(&driver)
            .map_err(|_| format!("Failed to get driver for {driver}"))?;
        let mut dataset = driver
            .create_vector_only(&name)
            .map_err(|_| format!("Failed to create dataset for file {name} with driver {name}"))?;
        dataset
            .flush_cache()
            .map_err(|_| "Failed to save new dataset to disc".to_string())?;
        Ok(WrappedDataset {
            file_name: name,
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
            srs: self
                .dataset
                .spatial_ref()
                .ok()
                .and_then(|srs| srs.to_wkt().ok()),
        })
    }

    pub fn set_spatial_ref(
        &mut self,
        spatial_ref: &gdal::spatial_ref::SpatialRef,
    ) -> gdal::errors::Result<()> {
        self.dataset.set_spatial_ref(spatial_ref)
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

use gdal::{
    raster::RasterBand,
    vector::{LayerAccess, LayerIterator},
    Dataset,
};
use itertools::Itertools;

use crate::{
    dataset_collection::{get_default_field_name, IndexedLayer},
    gdal_if::{LayerIndex, WrappedDataset},
    state::settings::GlobalSettings,
    FeatureInfo,
};

use super::{
    combined::StatefulLayerEnum,
    raster::{ImageType, RenderMethod, StatefulRasterBand, StatefulRasterInfo},
    shared::SharedInfo,
    vector::{StatefulVectorInfo, StatefulVectorLayer},
};

pub struct StatefulDataset {
    pub dataset: WrappedDataset,
    pub layer_index: Option<LayerIndex>,
    pub layer_info: Vec<StatefulVectorInfo>,
    pub band_info: Vec<StatefulRasterInfo>,
}

impl StatefulDataset {
    fn open(name: String, settings: &GlobalSettings) -> Result<Self, String> {
        Ok(Self::new(WrappedDataset::open(name)?, settings))
    }

    pub fn new(dataset: WrappedDataset, settings: &GlobalSettings) -> Self {
        let layer_count = dataset.dataset.layer_count();
        let band_count = dataset.dataset.rasterbands().count();

        let layer_info = dataset
            .dataset
            .layers()
            .map(|layer| StatefulVectorInfo {
                selected_feature: layer.feature(0).map(|_| 0),
                primary_field_name: get_default_field_name(&layer),
                shared: SharedInfo {
                    name: dataset.file_name.clone(),
                },
                display: false,
            })
            .collect_vec();

        // TODO: Make this customisable in settings
        let render_method = RenderMethod::Image;

        let band_info = dataset
            .dataset
            .rasterbands()
            .map(|_| StatefulRasterInfo {
                audio_settings: settings.get_default_audio().clone(),
                shared: SharedInfo {
                    name: dataset.file_name.clone(),
                },
                image_type: ImageType::default(),
                render: render_method,
                ocr: match render_method {
                    RenderMethod::GDAL => settings.default_ocr_for_gdal(),
                    RenderMethod::Image => true,
                },
            })
            .collect_vec();

        let layer_index = if layer_count > 0 {
            Some(LayerIndex::Vector(0))
        } else if band_count > 0 {
            Some(LayerIndex::Raster(1))
        } else {
            None
        };
        Self {
            dataset,
            layer_index,
            layer_info,
            band_info,
        }
    }

    fn from_raw(dataset: Dataset, name: String, settings: &GlobalSettings) -> Self {
        Self::new(WrappedDataset::wrap_existing(dataset, name), settings)
    }

    pub fn get_all_layers(&mut self) -> Vec<IndexedLayer> {
        self.dataset.get_all_layers()
    }

    pub fn layers_raw(&mut self) -> LayerIterator {
        self.dataset.dataset.layers()
    }

    pub fn layers(&mut self) -> impl Iterator<Item = StatefulVectorLayer<'_>> {
        self.dataset
            .vectors()
            .into_iter()
            .zip(&mut self.layer_info)
            .map(|(layer, info)| StatefulVectorLayer { layer, info })
    }

    pub fn raster_bands(&mut self) -> impl Iterator<Item = gdal::errors::Result<RasterBand>> {
        self.dataset.dataset.rasterbands()
    }

    pub fn get_current_layer(&mut self) -> Option<StatefulLayerEnum> {
        self.get_layer(self.layer_index?)
    }

    pub fn get_layer(&mut self, index: LayerIndex) -> Option<StatefulLayerEnum> {
        match index {
            LayerIndex::Vector(index) => self.get_vector(index).map(Into::into),
            LayerIndex::Raster(index) => self.get_raster(index).map(Into::into),
        }
    }

    pub fn get_vector(&mut self, idx: usize) -> Option<StatefulVectorLayer> {
        let layer = self.dataset.get_vector(idx);
        let info = self.layer_info.get_mut(idx);
        match (layer, info) {
            (Some(layer), Some(info)) => Some(StatefulVectorLayer { layer, info }),
            (None, None) => None,
            _ => panic!("Mismatch in gdal layers and stateful layer info"),
        }
    }

    pub fn get_raster(&mut self, idx: usize) -> Option<StatefulRasterBand> {
        let band = self.dataset.get_raster(idx);
        let info = self.band_info.get_mut(idx - 1);
        match (band, info) {
            (Some(band), Some(info)) => Some(StatefulRasterBand { band, info }),
            (None, None) => None,
            _ => panic!("Mismatch in gdal layers and stateful layer info"),
        }
    }

    pub fn get_current_feature(&mut self) -> Option<FeatureInfo> {
        let layer = self.get_current_layer()?;
        let layer = layer.as_vector()?;
        let feature = layer
            .layer
            .layer
            .feature(layer.info.selected_feature? as u64);
        feature.map(Into::into)
    }
}

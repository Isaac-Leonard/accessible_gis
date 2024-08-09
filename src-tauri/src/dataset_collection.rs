use std::slice::{Iter, IterMut};

use gdal::{
    raster::RasterBand,
    vector::{Layer, LayerAccess, LayerIterator},
    Dataset,
};
use itertools::Itertools;
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    gdal_if::{LayerEnum, LayerExt, LayerIndex, WrappedDataset, WrappedLayer, WrappedRasterBand},
    FeatureInfo, LayerDescriptor,
};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct StatefulVectorInfo {
    /// The index of each user selected feature for each layer of the dataset
    pub selected_feature: Option<usize>,
    /// The name of the field used to identify features
    pub primary_field_name: Option<String>,
    pub shared: SharedInfo,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct SharedInfo {
    pub display: bool,
}

pub struct StatefulVectorLayer<'a> {
    pub layer: WrappedLayer<'a>,
    pub info: &'a mut StatefulVectorInfo,
}

pub struct AudioSettings {
    pub min_freq: f64,
    pub max_freq: f64,
    pub volume: f64,
    no_data_value_sound: AudioIndicator,
    border_sound: AudioIndicator,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            min_freq: 220.0,
            max_freq: 880.0,
            volume: 1.0,
            no_data_value_sound: AudioIndicator::Different,
            border_sound: AudioIndicator::MinFreq,
        }
    }
}

#[derive(EnumIter)]
pub enum AudioIndicator {
    Silence,
    MinFreq,
    MaxFreq,
    Verbal,
    Different,
}

impl AudioIndicator {
    fn get_all_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}

pub struct StatefulRasterInfo {
    pub audio_settings: AudioSettings,
    pub shared: SharedInfo,
}

pub struct StatefulRasterBand<'a> {
    pub band: WrappedRasterBand<'a>,
    pub info: &'a mut StatefulRasterInfo,
}

pub enum StatefulLayerEnum<'a> {
    Raster(StatefulRasterBand<'a>),
    Vector(StatefulVectorLayer<'a>),
}

impl<'a> From<StatefulVectorLayer<'a>> for StatefulLayerEnum<'a> {
    fn from(v: StatefulVectorLayer<'a>) -> Self {
        Self::Vector(v)
    }
}

impl<'a> From<StatefulRasterBand<'a>> for StatefulLayerEnum<'a> {
    fn from(v: StatefulRasterBand<'a>) -> Self {
        Self::Raster(v)
    }
}

impl<'a> StatefulLayerEnum<'a> {
    pub fn shared_mut(&mut self) -> &mut SharedInfo {
        match self {
            Self::Vector(layer) => &mut layer.info.shared,
            Self::Raster(band) => &mut band.info.shared,
        }
    }

    pub fn as_vector(&self) -> Option<&StatefulVectorLayer> {
        if let Self::Vector(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_raster(&self) -> Option<&StatefulRasterBand> {
        if let Self::Raster(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn try_into_raster(self) -> Result<StatefulRasterBand<'a>, Self> {
        if let Self::Raster(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_vector(self) -> Result<StatefulVectorLayer<'a>, Self> {
        if let Self::Vector(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

pub struct StatefulDataset {
    pub dataset: WrappedDataset,
    pub layer_index: Option<LayerIndex>,
    pub layer_info: Vec<StatefulVectorInfo>,
    pub band_info: Vec<StatefulRasterInfo>,
}

impl StatefulDataset {
    fn open(name: String) -> Result<Self, String> {
        Ok(Self::new(WrappedDataset::open(name)?))
    }

    fn new(dataset: WrappedDataset) -> Self {
        let layer_count = dataset.dataset.layer_count();
        let band_count = dataset.dataset.rasterbands().count();

        let layer_info = dataset
            .dataset
            .layers()
            .map(|layer| StatefulVectorInfo {
                selected_feature: layer.feature(0).map(|_| 0),
                primary_field_name: get_default_field_name(&layer),
                shared: SharedInfo { display: false },
            })
            .collect_vec();

        let band_info = dataset
            .dataset
            .rasterbands()
            .map(|_| StatefulRasterInfo {
                audio_settings: AudioSettings::default(),
                shared: SharedInfo { display: false },
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

    fn from_raw(dataset: Dataset, name: String) -> Self {
        Self::new(WrappedDataset::wrap_existing(dataset, name))
    }

    pub fn get_all_layers(&mut self) -> Vec<IndexedLayer> {
        self.dataset.get_all_layers()
    }

    pub fn layers(&mut self) -> LayerIterator {
        self.dataset.dataset.layers()
    }

    pub fn raster_bands(&mut self) -> impl Iterator<Item = gdal::errors::Result<RasterBand>> {
        self.dataset.dataset.rasterbands()
    }

    pub fn get_current_layer(&mut self) -> Option<StatefulLayerEnum> {
        match self.layer_index {
            Some(LayerIndex::Vector(index)) => self.get_vector(index).map(Into::into),
            Some(LayerIndex::Raster(index)) => self.get_raster(index).map(Into::into),
            None => None,
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

pub struct NonEmptyDatasetCollection {
    datasets: Vec<StatefulDataset>,
    index: usize,
}

impl NonEmptyDatasetCollection {
    pub fn add(&mut self, dataset: StatefulDataset) {
        self.datasets.push(dataset)
    }

    pub fn add_gdal(&mut self, dataset: WrappedDataset) {
        self.datasets.push(StatefulDataset::new(dataset))
    }

    pub fn new(dataset: StatefulDataset) -> Self {
        Self {
            datasets: vec![dataset],
            index: 0,
        }
    }

    pub fn with_current_dataset_mut<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut StatefulDataset) -> T,
    {
        let dataset = &mut self.datasets[self.index];
        let res = f(dataset);
        dataset
            .dataset
            .save_changes()
            .expect("Could not flush changes to disc");
        res
    }

    pub fn create_from_current_dataset<E, F>(&mut self, f: F) -> Result<&mut StatefulDataset, E>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, E>,
    {
        let dataset = &mut self.datasets[self.index];
        let res = f(dataset)?;
        dataset
            .dataset
            .save_changes()
            .expect("Could not flush changes to disc");
        self.add(StatefulDataset::new(res));
        Ok(self.datasets.last_mut().unwrap())
    }

    pub fn iter(&mut self) -> std::slice::Iter<'_, StatefulDataset> {
        self.datasets.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, StatefulDataset> {
        self.datasets.iter_mut()
    }
}

#[derive(Default)]
pub enum DatasetCollection {
    #[default]
    Empty,
    NonEmpty(NonEmptyDatasetCollection),
}

impl DatasetCollection {
    pub fn create_from_current_dataset<E, F>(
        &mut self,
        f: F,
    ) -> Option<Result<&mut StatefulDataset, E>>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, E>,
    {
        match self {
            Self::Empty => None,
            Self::NonEmpty(datasets) => Some(datasets.create_from_current_dataset(f)),
        }
    }

    pub fn get_all_layers(&mut self) -> Vec<IndexedDatasetLayer> {
        let mut layers = Vec::new();
        for (ds_idx, dataset) in self.iter_mut().enumerate() {
            let ds_file = dataset.dataset.file_name.clone();
            for layer in dataset.get_all_layers() {
                layers.push(IndexedDatasetLayer {
                    layer,
                    dataset_index: ds_idx,
                    ds_file: ds_file.clone(),
                })
            }
        }
        layers
    }

    pub fn open(&mut self, name: String) -> Result<&mut StatefulDataset, String> {
        let dataset = WrappedDataset::open(name)?;
        Ok(self.add(StatefulDataset::new(dataset)))
    }

    pub fn set_index(&mut self, index: usize) -> Result<(), ()> {
        match self {
            Self::NonEmpty(datasets) => {
                datasets.index = index;
                Ok(())
            }
            Self::Empty => Err(()),
        }
    }

    pub fn new(dataset: StatefulDataset) -> Self {
        Self::NonEmpty(NonEmptyDatasetCollection::new(dataset))
    }

    pub fn add(&mut self, dataset: StatefulDataset) -> &mut StatefulDataset {
        if let Self::NonEmpty(datasets) = self {
            datasets.add(dataset);
            datasets.datasets.last_mut().unwrap()
        } else {
            *self = Self::NonEmpty(NonEmptyDatasetCollection::new(dataset));
            self.iter_mut().next().unwrap()
        }
    }

    pub fn with_current_dataset_mut<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        match self {
            Self::NonEmpty(datasets) => {
                let index = datasets.index;
                let dataset = &mut datasets.datasets[index];
                let res = f(dataset, index);
                dataset.dataset.save_changes();
                Some(res)
            }
            Self::Empty => None,
        }
    }

    pub fn iter(&mut self) -> Iter<'_, StatefulDataset> {
        match self {
            Self::NonEmpty(datasets) => datasets.iter(),
            Self::Empty => Iter::default(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, StatefulDataset> {
        match self {
            Self::NonEmpty(datasets) => datasets.iter_mut(),
            Self::Empty => IterMut::default(),
        }
    }
}

pub struct IndexedDatasetLayer<'a> {
    ds_file: String,
    dataset_index: usize,
    layer: IndexedLayer<'a>,
}

pub struct IndexedLayer<'a> {
    pub layer_index: usize,
    pub layer: LayerEnum<'a>,
}

impl From<IndexedDatasetLayer<'_>> for LayerDescriptor {
    fn from(value: IndexedDatasetLayer) -> Self {
        match value.layer.layer {
            LayerEnum::Layer(_layer) => Self {
                dataset: value.dataset_index,
                band: LayerIndex::Vector(value.layer.layer_index),
                dataset_file: value.ds_file,
            },
            LayerEnum::Band(band) => {
                let (_width, _length) = band.band().size();
                Self {
                    dataset: value.dataset_index,
                    band: LayerIndex::Raster(value.layer.layer_index),
                    dataset_file: value.ds_file,
                }
            }
        }
    }
}

fn get_default_field_name(layer: &Layer) -> Option<String> {
    let names = layer.get_field_names();
    let id = names.iter().find(|f| f.to_lowercase() == "id");
    if let Some(id) = id {
        return Some(id.clone());
    }
    let name = names.iter().find(|f| f.to_lowercase() == "name");
    if let Some(name) = name {
        return Some(name.clone());
    }
    let possible_name = names.iter().find(|f| f.to_lowercase().contains("name"));
    if let Some(possible_name) = possible_name {
        return Some(possible_name.clone());
    }
    return names.first().cloned();
}

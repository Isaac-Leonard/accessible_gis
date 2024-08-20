use std::slice::{Iter, IterMut};

use gdal::vector::Layer;

use crate::{
    gdal_if::{LayerEnum, LayerExt, LayerIndex, WrappedDataset},
    LayerDescriptor,
};

use super::gis::{
    combined::{DatasetLayerIndex, RasterIndex, StatefulLayerEnum, VectorIndex},
    dataset::StatefulDataset,
    raster::StatefulRasterBand,
    vector::StatefulVectorLayer,
};

pub struct NonEmptyDatasetCollection {
    datasets: Vec<StatefulDataset>,
    index: usize,
}

trait NonEmptyDelegator {
    fn get_non_empty(&self) -> Option<&NonEmptyDatasetCollection>;
    fn get_non_empty_mut(&mut self) -> Option<&mut NonEmptyDatasetCollection>;
}

pub trait NonEmptyDelegatorImpl: NonEmptyDelegator {
    fn get_current_index(&self) -> Option<DatasetLayerIndex> {
        let non_empty = self.get_non_empty()?;
        let dataset = non_empty.datasets.get(non_empty.index).unwrap();
        Some(DatasetLayerIndex {
            dataset: non_empty.index,
            layer: dataset.layer_index?,
        })
    }

    fn get_current_vector_index(&self) -> Option<VectorIndex> {
        let non_empty = self.get_non_empty()?;
        let dataset = non_empty.datasets.get(non_empty.index).unwrap();
        Some(VectorIndex {
            dataset: non_empty.index,
            layer: *dataset.layer_index?.as_vector()?,
        })
    }

    fn get_current_raster_index(&self) -> Option<RasterIndex> {
        let non_empty = self.get_non_empty()?;
        let dataset = non_empty.datasets.get(non_empty.index).unwrap();
        Some(RasterIndex {
            dataset: non_empty.index,
            band: *dataset.layer_index?.as_raster()?,
        })
    }

    fn get(&mut self, index: DatasetLayerIndex) -> Option<StatefulLayerEnum> {
        let dataset = self.get_non_empty_mut()?.datasets.get_mut(index.dataset)?;
        dataset.get_layer(index.layer)
    }

    fn get_vector(&mut self, index: VectorIndex) -> Option<StatefulVectorLayer> {
        let dataset = self.get_non_empty_mut()?.datasets.get_mut(index.dataset)?;
        dataset.get_vector(index.layer)
    }

    fn get_raster(&mut self, index: RasterIndex) -> Option<StatefulRasterBand> {
        let dataset = self.get_non_empty_mut()?.datasets.get_mut(index.dataset)?;
        dataset.get_raster(index.band)
    }
}

impl<T: NonEmptyDelegator> NonEmptyDelegatorImpl for T {}

impl NonEmptyDelegator for NonEmptyDatasetCollection {
    fn get_non_empty(&self) -> Option<&NonEmptyDatasetCollection> {
        Some(self)
    }
    fn get_non_empty_mut(&mut self) -> Option<&mut NonEmptyDatasetCollection> {
        Some(self)
    }
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

impl NonEmptyDelegator for DatasetCollection {
    fn get_non_empty(&self) -> Option<&NonEmptyDatasetCollection> {
        match self {
            Self::NonEmpty(ref datasets) => Some(datasets),
            Self::Empty => None,
        }
    }

    fn get_non_empty_mut(&mut self) -> Option<&mut NonEmptyDatasetCollection> {
        match self {
            Self::NonEmpty(ref mut datasets) => Some(datasets),
            Self::Empty => None,
        }
    }
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

pub fn get_default_field_name(layer: &Layer) -> Option<String> {
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

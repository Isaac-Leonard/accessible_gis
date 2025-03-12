use itertools::Itertools;

use crate::{
    dataset_collection::{DatasetCollection, NonEmptyDelegatorImpl},
    gdal_if::WrappedDataset,
};

use super::{
    gis::{
        combined::RasterIndex, dataset::StatefulDataset, raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
    settings::GlobalSettings,
};

#[derive(Default)]
pub struct UserState {
    pub datasets: DatasetCollection,
    pub raster_to_display: Option<RasterIndex>,
}

impl UserState {
    pub fn display_current_raster(&mut self) {
        self.raster_to_display = self.datasets.get_current_raster_index();
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
}

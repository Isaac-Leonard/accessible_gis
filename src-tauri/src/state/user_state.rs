use gdal::vector::LayerAccess;

use crate::{
    dataset_collection::{DatasetCollection, NonEmptyDelegatorImpl},
    gdal_if::{get_fields, WrappedDataset},
    tools::ToolList,
    FeatureInfo,
};

use super::{
    gis::{
        combined::{RasterIndex, VectorIndex},
        dataset::StatefulDataset,
        raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
    settings::GlobalSettings,
};

#[derive(Default)]
pub struct UserState {
    pub datasets: DatasetCollection,
    pub tools_data: ToolList,
    pub raster_to_display: Option<RasterIndex>,
    pub vector_to_display: Option<VectorIndex>,
}

impl UserState {
    pub fn display_current_raster(&mut self) {
        self.raster_to_display = self.datasets.get_current_raster_index();
    }

    pub fn display_current_vector(&mut self) {
        self.vector_to_display = self.datasets.get_current_vector_index();
    }

    pub fn get_raster_to_display(&mut self) -> Option<StatefulRasterBand> {
        self.datasets.get_raster(self.raster_to_display?)
    }

    pub fn get_vector_to_display(&mut self) -> Option<StatefulVectorLayer> {
        self.datasets.get_vector(self.vector_to_display?)
    }

    pub fn get_vector_index_to_display(&mut self) -> Option<VectorIndex> {
        self.vector_to_display
    }

    pub fn get_raster_index_to_display(&mut self) -> Option<RasterIndex> {
        self.raster_to_display
    }

    pub fn get_vectors_for_display(&mut self) -> Vec<FeatureInfo> {
        let main_srs = self
            .with_current_dataset_mut(|ds, _| ds.dataset.dataset.spatial_ref().unwrap())
            .unwrap();
        self.datasets
            .iter_mut()
            // .filter(|ds| ds.display)
            .filter_map(|ds| ds.get_current_layer()?.try_into_vector().ok())
            .flat_map(|mut layer| {
                layer
                    .layer
                    .layer()
                    .features()
                    .map(|feature| {
                        let geom = feature.geometry().unwrap().transform_to(&main_srs).unwrap();
                        FeatureInfo::new(geom.to_geo().unwrap().into(), get_fields(&feature))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
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

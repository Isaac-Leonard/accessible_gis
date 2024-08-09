use gdal::vector::LayerAccess;

use crate::{
    dataset_collection::{DatasetCollection, StatefulDataset},
    gdal_if::{get_fields, WrappedDataset},
    tools::ToolList,
    FeatureInfo,
};

#[derive(Default)]
pub struct UserState {
    pub datasets: DatasetCollection,
    pub tools_data: ToolList,
    pub show_towns: bool,
}

impl UserState {
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
    ) -> Option<Result<&mut StatefulDataset, E>>
    where
        F: FnOnce(&mut StatefulDataset) -> Result<WrappedDataset, E>,
    {
        self.datasets.create_from_current_dataset(f)
    }

    pub fn with_current_dataset_mut<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut StatefulDataset, usize) -> T,
    {
        self.datasets.with_current_dataset_mut(f)
    }
}

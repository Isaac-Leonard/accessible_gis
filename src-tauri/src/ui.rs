use gdal::vector::LayerAccess;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    gdal_if::{FieldSchema, FieldValue, LayerExt, LayerIndex},
    geometry::Point,
    state::{settings::GlobalSettings, AppData},
    FeatureInfo, LayerDescriptor,
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
#[serde(tag = "name")]
pub enum UiScreen {
    Layers(LayerScreen),
    ThiessenPolygons,
    NewDataset(NewDatasetScreenData),
    Settings(GlobalSettings),
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct NewDatasetScreenData {
    pub drivers: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
#[serde(tag = "type")]
pub enum UiToolData {
    TracingGeometry {
        points: Vec<Point>,
        geometrys_count: usize,
    },
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct LayerScreen {
    pub layers: Vec<LayerDescriptor>,
    pub layer_info: Option<LayerScreenInfo>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
#[serde(tag = "type")]
pub enum LayerScreenInfo {
    Vector(VectorScreenData),
    Raster(RasterScreenData),
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct VectorScreenData {
    pub field_schema: Vec<FieldSchema>,
    pub features: Vec<FeatureIdentifier>,
    pub feature: Option<FeatureInfo>,
    pub srs: Option<String>,
    pub editable: bool,
    pub layer_index: usize,
    pub dataset_index: usize,
    pub display: bool,
    pub name_field: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct FeatureIdentifier {
    pub name: Option<String>,
    fid: u64,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct RasterScreenData {
    pub layer_index: usize,
    pub dataset_index: usize,
    pub cols: usize,
    pub rows: usize,
    pub srs: Option<String>,
    pub tool: Option<UiToolData>,
    pub display: bool,
}

impl AppData {
    pub fn get_layers_screen(&mut self) -> LayerScreen {
        let layers = self
            .shared
            .datasets
            .get_all_layers()
            .into_iter()
            .map_into()
            .collect_vec();
        let layer_info = self
            .shared
            .with_current_dataset_mut(|ds, ds_index| match ds.layer_index {
                Some(LayerIndex::Vector(index)) => {
                    let feature = ds.get_current_feature();
                    let mut layer = ds.get_vector(index).expect("Failed to get vector layer");
                    let primary_field_name = layer.info.primary_field_name.as_ref();
                    let features = layer
                        .layer
                        .layer
                        .features()
                        .map(move |feature| FeatureIdentifier {
                            name: primary_field_name
                                .and_then(|name| feature.field(name).unwrap())
                                .map(|x| FieldValue::from(x).to_string()),
                            fid: feature.fid().unwrap(),
                        })
                        .collect_vec();
                    Some(LayerScreenInfo::Vector(VectorScreenData {
                        name_field: primary_field_name.cloned(),
                        display: layer.info.shared.display,
                        dataset_index: ds_index,
                        srs: try { layer.layer.layer.spatial_ref()?.to_wkt().ok()? },
                        field_schema: layer.layer.get_field_schema(),
                        features,
                        feature,
                        editable: ds.dataset.editable,
                        layer_index: index,
                    }))
                }
                Some(LayerIndex::Raster(index)) => {
                    let band = ds.get_raster(index).unwrap();
                    let (cols, rows) = band.band.band().size();
                    Some(LayerScreenInfo::Raster(RasterScreenData {
                        dataset_index: ds_index,
                        layer_index: index,
                        cols,
                        rows,
                        srs: band.band.srs.clone(),
                        tool: None,
                        display: band.info.shared.display,
                    }))
                }
                None => None,
            })
            .flatten();
        LayerScreen { layers, layer_info }
    }
}

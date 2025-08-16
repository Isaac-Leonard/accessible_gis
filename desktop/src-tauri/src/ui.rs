use std::path::PathBuf;

use gdal::{Metadata, vector::LayerAccess};
use itertools::Itertools;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};

use crate::{
    FeatureInfo,
    commands::SortOption,
    dataset_collection::NonEmptyDelegatorImplExt,
    errors::ApplicationError,
    gdal_if::{DatasetMetadata, FieldSchema, FieldValue, LayerExt, LayerIndex},
    state::{
        AppData,
        configurable_tools::{Input, SavedToolOutputAction},
        gis::{combined::RasterIndex, raster::RenderMethod},
        settings::{AudioSettings, GlobalSettings},
    },
    tools::shape_analysis::FloatWrapper,
};

#[derive(Clone, Serialize, PartialEq, Debug, specta::Type)]
pub struct UiState {
    pub screen: UiScreen,
    pub errors: Vec<ApplicationError>,
    pub tool_outputs: Vec<SavedToolOutputAction>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
#[serde(tag = "name")]
pub enum UiScreen {
    Project(ProjectScreen),
    ThiessenPolygons,
    NewDataset(NewDatasetScreenData),
    Settings(GlobalSettings),
    TouchDevice(TouchDeviceState),
    Errors,
    Tools(ToolsScreenInfo),
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct NewDatasetScreenData {
    pub drivers: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, Default, specta::Type)]
#[serde(tag = "type")]
pub enum ProjectScreen {
    Project(ProjectScreenInfo),
    #[default]
    NotLoaded,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct ProjectScreenInfo {
    pub layers: Vec<LayerDescriptor>,
    pub layer_info: Option<LayerScreenInfo>,
    pub ip: String,
    prefered_display_fields: Vec<String>,
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
    pub sort_features_by: SortOption,
    pub metadata: VectorScreenMetadata,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct VectorScreenMetadata {
    srs: Option<String>,
    other: DatasetMetadata,
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
    pub display: bool,
    pub render_method: RenderMethod,
    pub ocr: bool,
    pub audio_settings: AudioSettings,
    pub metadata: RasterScreenMetadata,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct RasterScreenMetadata {
    pub cols: usize,
    pub rows: usize,
    pub srs: Option<String>,
    pub other: DatasetMetadata,
}

impl AppData {
    pub fn get_project_screen_info(&mut self) -> Option<ProjectScreenInfo> {
        self.with_project_fallible(|project| {
            let layers = project
                .datasets
                .get_all_layers()?
                .into_iter()
                .map_into()
                .collect_vec();
            let visible_raster_index = project.get_raster_index_to_display();
            let layer_info = project
                .with_current_dataset_mut(|ds, ds_index| match ds.layer_index {
                    Some(LayerIndex::Vector(index)) => {
                        let feature = ds.get_current_feature();
                        let mut layer = ds.get_vector(index).expect("Failed to get vector layer");
                        let primary_field_name = layer.info.primary_field_name.as_ref();

                        let features = match &layer.info.sort_features_by {
                            SortOption::Default => layer.layer.layer.features().collect_vec(),
                            SortOption::Field(field) => {
                                let index = layer.layer.get_field_index(&field).unwrap();
                                layer
                                    .layer
                                    .layer
                                    .features()
                                    .sorted_by_key(|feature| {
                                        feature.field(index).unwrap().map(FieldValue::from)
                                    })
                                    .collect_vec()
                            }
                            SortOption::Area => layer
                                .layer
                                .layer
                                .features()
                                .sorted_by_key(|feature| {
                                    feature.geometry().map(|geom| FloatWrapper(geom.area()))
                                })
                                .collect_vec(),
                        };
                        let features = features
                            .into_iter()
                            .map(move |feature| FeatureIdentifier {
                                name: primary_field_name
                                    .and_then(|name| {
                                        feature.field(feature.field_index(name).unwrap()).unwrap()
                                    })
                                    .map(|x| FieldValue::from(x).to_string()),
                                fid: feature.fid().unwrap(),
                            })
                            .collect_vec();
                        let srs = layer
                            .layer
                            .layer
                            .spatial_ref()
                            .and_then(|x| x.to_wkt().ok());
                        Ok(Some(LayerScreenInfo::Vector(VectorScreenData {
                            name_field: primary_field_name.cloned(),
                            display: layer.info.display,
                            dataset_index: ds_index,
                            srs: srs.clone(),
                            field_schema: layer.layer.get_field_schema(),
                            features,
                            feature: feature.transpose()?,
                            layer_index: index,
                            sort_features_by: layer.info.sort_features_by.clone(),
                            editable: ds.dataset.editable,
                            metadata: VectorScreenMetadata {
                                srs,
                                other: ds.dataset.get_metadata(),
                            },
                        })))
                    }
                    Some(LayerIndex::Raster(index)) => {
                        let band = ds.get_raster(index).unwrap();
                        let (cols, rows) = band.band.band().size();
                        Ok(Some(LayerScreenInfo::Raster(RasterScreenData {
                            dataset_index: ds_index,
                            layer_index: index,
                            display: visible_raster_index
                                == Some(RasterIndex {
                                    dataset: ds_index,
                                    band: index,
                                }),
                            render_method: band.info.render,
                            ocr: band.info.ocr,
                            audio_settings: band.info.audio_settings.clone(),
                            metadata: RasterScreenMetadata {
                                cols,
                                rows,
                                srs: band
                                    .band
                                    .srs
                                    .clone()
                                    .map(|srs| srs.to_pretty_wkt().unwrap()),
                                other: ds.dataset.get_metadata(),
                            },
                        })))
                    }
                    None => Ok(None),
                })
                .transpose()?
                .flatten();
            let port = 80;
            Ok(ProjectScreenInfo {
                layers,
                layer_info,
                ip: local_ip()
                    .map(|ip| format!("Server running at http://{}:{}/", ip, port))
                    .unwrap_or_else(|e| {
                        format!("Unable to get local IP address, got error: {}", e)
                    }),
                prefered_display_fields: project.prefered_display_fields.clone(),
            })
        })
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, specta::Type)]
pub struct LayerDescriptor {
    pub dataset: usize,
    #[serde(flatten)]
    pub band: LayerIndex,
    pub dataset_file: PathBuf,
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
pub struct TouchDeviceState {
    pub use_labels: bool,
    pub announce_leaving: bool,
    pub announce_geometry_type: bool,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolsScreenInfo {
    pub tools: Vec<ToolDescriptor>,
    pub layers: Vec<LayerDescriptor>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolDescriptor {
    pub label: String,
    pub inputs: Vec<Input>,
}

use super::app::BasicApp;
use super::events::{dispatch_action, Action};
use crate::audio::{get_audio, AudioMessage};
use crate::gis::commands::slope_of_dataset;
use crate::gis::raster::RasterIndex;

use super::new_dataset_window::create_dataset;
use super::raster::*;
use super::vector::{get_fields, FeatureViewProps, VectorLayerProps, VectorLayerView};
use cacao::appkit::App;
use cacao::filesystem::{FileSavePanel, FileSelectPanel};
use cacao::foundation::NSURL;

use cacao_framework::{Component, Message, VButton, VComponent, VLabel, VNode};
use gdal::errors::GdalError;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic;
use std::sync::mpsc::SyncSender;

struct SendableDataset(Dataset);

unsafe impl Send for SendableDataset {}

#[derive(Clone, PartialEq)]
pub struct MainView;
impl Component for MainView {
    type Props = ();
    type State = Vec<DatasetWrapper>;
    type Message = Action;
    fn render(_props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Button(VButton {
                    text: "Select file".to_owned(),
                    click: Some(|_, _| FileSelectPanel::new().show(file_selection_handler)),
                }),
            ),
            (
                1,
                VNode::Button(VButton {
                    text: "New dataset".to_owned(),
                    click: Some(|_, _| dispatch_action(Action::OpenNewDatasetWindow)),
                }),
            ),
        ]
        .into_iter()
        .chain(state.iter().enumerate().map(|(index, dataset)| {
            (
                index + 10,
                VNode::Custom(VComponent::new::<DatasetView, BasicApp>(DatasetViewProps {
                    index,
                    dataset: dataset.clone(),
                })),
            )
        }))
        .collect()
    }

    fn on_message(msg: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        eprintln!("{:?}", msg);
        match msg {
            Action::GotFile(path) => state.push(DatasetWrapper::try_from(path.clone()).unwrap()),
            Action::CreateDataset(settings) => {
                state.push(DatasetWrapper::from((
                    PathBuf::from_str(&settings.file_name).unwrap(),
                    create_dataset(settings).unwrap(),
                )));
            }
            Action::CopyDataset(index) => {
                let index = *index;
                let mut panel = FileSavePanel::new();
                panel.set_message("Destination of copy:");
                panel.show(move |path| {
                    if let Some(path) = path {
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            Action::CreateCoppiedDataset(index, path.into()),
                        ));
                    }
                })
            }
            Action::CreateCoppiedDataset(index, path) => {
                let copy = {
                    let dataset = &state[*index].dataset();
                    dataset.create_copy(&dataset.driver(), path, &[]).unwrap()
                };
                state.push(DatasetWrapper::from((path.to_owned(), copy)));
            }
            Action::SlopeRaster(index) => {
                let index = *index;
                let mut panel = FileSavePanel::new();
                panel.set_message("Destination of slope data:");
                panel.show(move |path| {
                    if let Some(path) = path {
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            Action::CreateSlopeRaster(index, path.into()),
                        ));
                    }
                })
            }
            Action::CreateSlopeRaster(ref index, ref path) => {
                let dest: PathBuf = path.clone();
                let src = state[index.dataset].path();
                std::thread::spawn(move || {
                    // TODO: Check output for success
                    let _slope = slope_of_dataset(src, dest.clone());
                    App::<BasicApp, Message>::dispatch_main(Message::custom(Action::GotFile(
                        dest.clone(),
                    )));
                });
            }
            _ => (),
        }
        true
    }
}

#[derive(Clone, PartialEq)]
pub struct DatasetView;

#[derive(Clone)]
pub struct DatasetViewState(pub SyncSender<AudioMessage>);
impl Default for DatasetViewState {
    fn default() -> Self {
        Self(get_audio())
    }
}

impl PartialEq for DatasetViewState {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq)]
pub struct DatasetViewProps {
    pub dataset: DatasetWrapper,
    pub index: usize,
}
impl DatasetViewProps {
    fn dataset(&self) -> &Dataset {
        self.dataset.dataset()
    }
}
#[derive(Clone)]
pub struct DatasetWrapper {
    dataset: Rc<Dataset>,
    path: PathBuf,
    id: usize,
}

impl DatasetWrapper {
    fn dataset(&self) -> &Dataset {
        self.dataset.as_ref()
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl TryFrom<PathBuf> for DatasetWrapper {
    type Error = GdalError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            id: gen_id(),
            path: value.clone(),
            dataset: Rc::new(Dataset::open(value)?),
        })
    }
}

impl From<(PathBuf, Dataset)> for DatasetWrapper {
    /// You must ensure that path is the correct path for this dataset
    /// TODO: Should this be unsafe?
    /// Unsafety is normally about pointers and memory but attaching the wrong file to a file path feels pretty unsafe to me
    fn from(value: (PathBuf, Dataset)) -> Self {
        Self {
            id: gen_id(),
            path: value.0,
            dataset: Rc::new(value.1),
        }
    }
}

impl PartialEq for DatasetWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Component for DatasetView {
    type Props = DatasetViewProps;
    type State = DatasetViewState;
    type Message = Action;
    fn render(
        props: &Self::Props,
        _state: &Self::State,
    ) -> Vec<(usize, cacao_framework::VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: format!("Driver: {}", props.dataset().driver().long_name(),),
                }),
            ),
            (
                1,
                VNode::Label(VLabel {
                    text: props
                        .dataset()
                        .spatial_ref()
                        .map(|x| x.to_pretty_wkt().unwrap())
                        .unwrap_or_else(|_| {
                            props
                                .dataset()
                                .layer(0)
                                .map(|x| x.spatial_ref().unwrap().to_pretty_wkt().unwrap())
                                .unwrap_or_else(|_| {
                                    "No spatial reference could be found".to_owned()
                                })
                        }),
                }),
            ),
            (
                2,
                VNode::Button(VButton {
                    text: "Copy dataset".to_owned(),
                    click: Some(|props: &Self::Props, _: &mut Self::State| {
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            Action::CopyDataset(props.index),
                        ))
                    }),
                }),
            ),
        ]
        .into_iter()
        .chain(
            props
                .dataset()
                .layers()
                .enumerate()
                .map(|(index, mut layer)| {
                    (
                        index + 20,
                        VNode::Custom(VComponent::new::<VectorLayerView, BasicApp>(
                            VectorLayerProps {
                                labeled_by: None,
                                common_fields: get_fields(&mut layer),
                                feature_props: layer
                                    .features()
                                    .map(|feature| FeatureViewProps {
                                        labeled_by: None,
                                        feature: feature.into(),
                                        position: index,
                                    })
                                    .collect(),
                            },
                        )),
                    )
                }),
        )
        .chain({
            let mut rasters = Vec::new();
            for i in 0..props.dataset().raster_count() {
                let band = props.dataset().rasterband(i + 1).unwrap();
                rasters.push(band);
            }
            rasters.into_iter().enumerate().map(|(index, band)| {
                (
                    index + props.dataset().layer_count() as usize + 21,
                    VNode::Custom(VComponent::new::<RasterLayerView, BasicApp>(
                        RasterLayerProps::new(band, RasterIndex::new(props.index, index + 1)),
                    )),
                )
            })
        })
        .collect()
    }
    fn on_message(message: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        if let Action::SendAudioGraph(graph, settings) = message {
            state
                .0
                .send(AudioMessage::PlayHistogram(
                    graph.clone(),
                    settings.clone(),
                    Default::default(),
                ))
                .expect("Something went wrong in the audio thread")
        };
        false
    }
}

fn file_selection_handler(paths: Vec<NSURL>) {
    if paths.is_empty() {
        // User canceled
        return;
    }
    let path = paths[0].pathbuf();
    // Simplistic check for vector file
    if path.is_dir() {
        dispatch_action(Action::InvalidFile(path));
        return;
    }
    App::<BasicApp, Message>::dispatch_main(Message::custom(Action::GotFile(path)))
}

fn gen_id() -> usize {
    static COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
    COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
}

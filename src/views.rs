use crate::app::BasicApp;
use crate::audio::{get_audio, AudioMessage};
use crate::events::{dispatch_action, Action};

use crate::new_dataset_window::create_dataset;
use crate::raster::*;
use crate::vector::{get_fields, FeatureViewProps, VectorLayerProps, VectorLayerView};
use cacao::appkit::App;
use cacao::filesystem::FileSelectPanel;
use cacao::foundation::NSURL;

use cacao_framework::{Component, Message, VButton, VComponent, VLabel, VNode};
use gdal::errors::GdalError;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic;
use std::sync::mpsc::Sender;

#[derive(Clone, PartialEq)]
pub struct MainView;
impl Component for MainView {
    type Props = ();
    type State = Vec<DatasetViewProps>;
    type Message = Action;
    fn render(props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
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
                VNode::Custom(VComponent::new::<DatasetView, BasicApp>(dataset.clone())),
            )
        }))
        .collect()
    }

    fn on_message(msg: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        match msg {
            &Action::GotFile(ref path) => {
                state.push(DatasetViewProps::try_from(path.clone()).unwrap())
            }
            &Action::CreateDataset(ref settings) => {
                state.push(DatasetViewProps::from(create_dataset(settings).unwrap()));
            }
            _ => (),
        }
        true
    }
}

#[derive(Clone, PartialEq)]
pub struct DatasetView;

#[derive(Clone)]
pub struct DatasetViewState(pub Sender<AudioMessage>);
impl Default for DatasetViewState {
    fn default() -> Self {
        Self(get_audio())
    }
}

impl PartialEq for DatasetViewState {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct DatasetViewProps {
    pub dataset: Rc<Dataset>,
    pub id: usize,
}

impl TryFrom<PathBuf> for DatasetViewProps {
    type Error = GdalError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            id: gen_id(),
            dataset: Rc::new(Dataset::open(value)?),
        })
    }
}
impl From<Dataset> for DatasetViewProps {
    fn from(value: Dataset) -> Self {
        Self {
            id: gen_id(),
            dataset: value.into(),
        }
    }
}
impl PartialEq for DatasetViewProps {
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
        state: &Self::State,
    ) -> Vec<(usize, cacao_framework::VNode<Self>)> {
        vec![(
            0,
            VNode::Label(VLabel {
                text: props
                    .dataset
                    .spatial_ref()
                    .map(|x| x.to_pretty_wkt().unwrap())
                    .unwrap_or_else(|_| {
                        props
                            .dataset
                            .layer(0)
                            .map(|x| x.spatial_ref().unwrap().to_pretty_wkt().unwrap())
                            .unwrap_or_else(|_| "No spacial reference could be found".to_owned())
                    }),
            }),
        )]
        .into_iter()
        .chain(
            props
                .dataset
                .layers()
                .enumerate()
                .map(|(index, mut layer)| {
                    (
                        index + 1,
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
            for i in 0..props.dataset.raster_count() {
                let band = props.dataset.rasterband(i + 1).unwrap();
                rasters.push(band);
            }
            rasters.into_iter().enumerate().map(|(index, band)| {
                (
                    index + props.dataset.layer_count() as usize + 1,
                    VNode::Custom(VComponent::new::<RasterLayerView, BasicApp>(
                        (band, index).into(),
                    )),
                )
            })
        })
        .collect()
    }
    fn on_message(message: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        match message {
            Action::SendAudioGraph(graph, settings) => state
                .0
                .send(AudioMessage::PlayHistogram(graph.clone(), settings.clone()))
                .unwrap(),
            _ => {}
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

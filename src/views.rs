use crate::app::BasicApp;
use crate::audio::{get_audio, AudioMessage};
use crate::events::{dispatch_action, Action};

use crate::raster::*;
use crate::vector::{FeatureViewProps, VectorLayerProps, VectorLayerView};
use cacao::appkit::App;
use cacao::filesystem::FileSelectPanel;
use cacao::foundation::NSURL;

use cacao_framework::{Component, Message, VButton, VComponent, VLabel, VNode};
use gdal::vector::LayerAccess;
use gdal::Dataset;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Clone, PartialEq)]
pub struct MainView;
impl Component for MainView {
    type Props = ();
    type State = Option<PathBuf>;
    type Message = Action;
    fn render(props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        if let Some(path) = state {
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
                    VNode::Custom(VComponent::new::<DatasetView, BasicApp>(DatasetViewProps {
                        file: path.clone(),
                        dataset: Dataset::open(path).unwrap(),
                    })),
                ),
            ]
        } else {
            vec![(
                0,
                VNode::Button(VButton {
                    text: "Select file".to_owned(),
                    click: Some(|_, _| FileSelectPanel::new().show(file_selection_handler)),
                }),
            )]
        }
    }
    fn on_message(msg: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        if let &Action::GotFile(ref path) = msg {
            *state = Some(path.clone())
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

pub struct DatasetViewProps {
    dataset: Dataset,
    file: PathBuf,
}
impl Clone for DatasetViewProps {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            dataset: Dataset::open(&self.file).unwrap(),
        }
    }
}

impl PartialEq for DatasetViewProps {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file
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
                    .unwrap_or_else(|_| props.dataset.layer(0).unwrap().spatial_ref().unwrap())
                    .to_pretty_wkt()
                    .unwrap(),
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
                                common_fields: Vec::new(),
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

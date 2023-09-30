use crate::app::BasicApp;
use crate::audio::{get_audio, AudioMessage};
use crate::events::{dispatch_action, Action};
use crate::graph::{generate_image_histogram, HistogramSettings, RasterGraphSettings};

use cacao::appkit::App;
use cacao_framework::{Component, Message, VButton, VComponent, VLabel, VList, VNode};
use gdal::raster::GdalDataType;

use gdal::raster::{RasterBand, StatisticsAll};
use ndarray::Array2;

use std::sync::mpsc::Sender;

#[derive(PartialEq)]
pub struct RasterViewerData {
    width: usize,
    height: usize,
    stats: StatisticsAll,
}

impl Clone for RasterViewerData {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            stats: StatisticsAll {
                min: self.stats.min,
                max: self.stats.max,
                mean: self.stats.mean,
                std_dev: self.stats.std_dev,
            },
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct RasterLayerProps {
    data: RawRasterData,
    position: usize,
    band_type: GdalDataType,
    hist: Option<Vec<f64>>,
    min: f64,
    max: f64,
    stats: RasterViewerData,
}

#[derive(PartialEq, Clone)]
pub struct RasterLayerView;

impl Component for RasterLayerView {
    type Props = RasterLayerProps;
    type State = ();
    fn render(props: &Self::Props, _: &Self::State) -> Vec<(usize, VNode<Self>)> {
        let main = VNode::Custom(VComponent::new::<AudioControls, BasicApp>(
            props.data.clone(),
        ));
        let stats = VNode::Custom(VComponent::new::<StatsComponent, BasicApp>(
            props.stats.clone(),
        ));

        if let Some(hist) = &props.hist {
            vec![
                (0, main),
                (
                    1,
                    VNode::Custom(VComponent::new::<HistComponent, BasicApp>((
                        hist.clone(),
                        props.position,
                    ))),
                ),
                (2, stats),
            ]
        } else {
            vec![(0, main), (2, stats)]
        }
    }
}

impl<'a> From<(RasterBand<'a>, usize)> for RasterLayerProps {
    fn from((band, position): (RasterBand, usize)) -> Self {
        let band_type = band.band_type();
        let hist = match band_type {
            GdalDataType::UInt8 => Some(generate_image_histogram(
                band.read_band_as::<u8>().unwrap().data,
            )),
            _ => None,
        };
        let data: Array2<f64> = match band_type {
            GdalDataType::UInt8 => band
                .read_as_array::<u8>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),
            GdalDataType::UInt16 => band
                .read_as_array::<u16>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),

            GdalDataType::UInt32 => band
                .read_as_array::<u32>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),

            GdalDataType::Int8 => band
                .read_as_array::<i8>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),

            GdalDataType::Int16 => band
                .read_as_array::<i16>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),

            GdalDataType::Int32 => band
                .read_as_array::<i32>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),

            GdalDataType::Float32 => band
                .read_as_array::<f32>((0, 0), band.size(), band.size(), None)
                .unwrap()
                .mapv_into_any(|x| x as f64),

            GdalDataType::Float64 => band
                .read_as_array::<f64>((0, 0), band.size(), band.size(), None)
                .unwrap(),

            _ => panic!("Unknown datatype in raster band"),
        };
        let min_max = band.compute_raster_min_max(false).unwrap();
        RasterLayerProps {
            band_type,
            min: min_max.min,
            max: min_max.max,
            data: RawRasterData {
                position,
                data_type: band_type.name(),
                data,
                min: min_max.min,
                max: min_max.max,
                no_data_value: band.no_data_value(),
                sender: get_audio(),
            },
            hist,
            position,
            stats: RasterViewerData {
                stats: band.get_statistics(true, true).unwrap().unwrap(),
                width: band.size().0,
                height: band.size().1,
            },
        }
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct ImagePoint {
    x: usize,
    y: usize,
}

#[derive(Clone, PartialEq)]
pub struct StatsComponent;

impl Component for StatsComponent {
    type Props = RasterViewerData;
    type State = ImagePoint;
    fn render(
        RasterViewerData {
            width,
            height,
            stats,
        }: &Self::Props,
        ImagePoint { x, y }: &Self::State,
    ) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: stats.mean.to_string(),
                }),
            ),
            (
                1,
                VNode::Label(VLabel {
                    text: format!("x: {x}, y: {y}, width: {width}, height: {height}",),
                }),
            ),
            (
                2,
                VNode::Label(VLabel {
                    text: format!("min: {}, max: {}", stats.min, stats.max),
                }),
            ),
            (
                3,
                VNode::Button(VButton {
                    click: Some(|data, point| point.y += data.height),
                    text: "Move North".to_string(),
                }),
            ),
            (
                4,
                VNode::Button(VButton {
                    click: Some(|data, point| point.y = point.y.saturating_sub(data.height)),
                    text: "Move south".to_string(),
                }),
            ),
        ]
    }
}

#[derive(Clone, PartialEq)]
pub struct AudioControls;
impl Component for AudioControls {
    type Props = RawRasterData;
    type State = RasterGraphSettings;
    type Message = RasterGraphSettings;
    fn render(props: &Self::Props, _state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: format!("Raster file with {} data", props.data_type),
                }),
            ),
            (
                1,
                VNode::Button(VButton {
                    text: "Play graph".to_string(),
                    click: Some(|data, settings| {
                        data.sender.send(AudioMessage::PlayRaster(
                            data.data.clone(),
                            data.min,
                            data.max,
                            data.no_data_value,
                            settings.clone(),
                        ));
                    }),
                }),
            ),
            (
                2,
                VNode::Button(VButton {
                    text: "Change raster audiograph settings".to_owned(),
                    click: Some(|props, settings| {
                        dispatch_action(Action::SendChangeRasterGraphSettings(
                            props.position,
                            settings.clone(),
                        ))
                    }),
                }),
            ),
        ]
    }

    fn on_message(msg: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        *state = msg.clone();
        false
    }
}

#[derive(Clone)]
pub struct RawRasterData {
    pub position: usize,
    pub data_type: String,
    pub data: Array2<f64>,
    pub min: f64,
    pub max: f64,
    pub no_data_value: Option<f64>,
    pub sender: Sender<AudioMessage>,
}

impl PartialEq for RawRasterData {
    fn eq(&self, other: &Self) -> bool {
        self.data_type == other.data_type
            && self.no_data_value == other.no_data_value
            && self.data == other.data
            && self.min == other.min
            && self.max == other.max
    }
}

fn render_histagram_row(
    index: usize,
    props: &(Vec<f64>, usize),
    _: &HistogramSettings,
) -> Vec<VNode<HistComponent>> {
    vec![VNode::Label(VLabel {
        text: props.0[index].to_string(),
    })]
}

#[derive(Clone, PartialEq)]
pub struct HistComponent;
impl Component for HistComponent {
    type Props = (Vec<f64>, usize);
    type State = HistogramSettings;
    type Message = HistogramSettings;
    fn render(props: &Self::Props, _state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Button(VButton {
                    text: "Play histogram".to_owned(),
                    click: Some(|(hist, _), settings| {
                        dispatch_action(Action::SendAudioGraph(hist.clone(), settings.clone()))
                    }),
                }),
            ),
            (
                1,
                VNode::List(VList {
                    count: props.0.len(),
                    render: render_histagram_row,
                }),
            ),
            (
                2,
                VNode::Button(VButton {
                    text: "Change histogram settings".to_owned(),
                    click: Some(|(_, position), settings| {
                        dispatch_action(Action::SendChangeHistogramSettings(
                            *position,
                            settings.clone(),
                        ))
                    }),
                }),
            ),
        ]
    }

    fn on_message(msg: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        *state = msg.clone();
        // Only changes audio stuff, the actual ui stays the same, for now
        false
    }
}

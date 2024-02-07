use super::app::BasicApp;
use super::events::{dispatch_action, Action};
use super::histogram_settings_window::HistogramSettingsWrapper;
use crate::audio::graph::{generate_image_histogram, HistogramSettings, RasterGraphSettings};
use crate::audio::AudioMessage;
use crate::raster::{RasterIndex, RawRasterData};

use cacao::appkit::App;
use cacao_framework::{Component, Message, VButton, VComponent, VLabel, VList, VNode};
use gdal::raster::{GdalDataType, StatisticsMinMax};

use gdal::raster::RasterBand;

use std::f64::NAN;

#[derive(PartialEq)]
pub struct RasterViewerData {
    pub width: usize,
    pub height: usize,
    pub stats: StatisticsMinMax,
    pub no_data_value: f64,
}

impl Clone for RasterViewerData {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            stats: StatisticsMinMax {
                min: self.stats.min,
                max: self.stats.max,
            },
            no_data_value: self.no_data_value,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct RasterLayerProps {
    data: RawRasterData,
    index: RasterIndex,
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
                        props.index,
                    ))),
                ),
                (2, stats),
            ]
        } else {
            vec![(0, main), (2, stats)]
        }
    }
}

impl RasterLayerProps {
    pub fn new(band: RasterBand, index: RasterIndex) -> Self {
        let band_type = band.band_type();
        let hist = match band_type {
            GdalDataType::UInt8 => Some(generate_image_histogram(
                band.read_band_as::<u8>().unwrap().data,
            )),
            _ => None,
        };
        let min_max = band.compute_raster_min_max(false).unwrap();
        RasterLayerProps {
            band_type,
            min: min_max.min,
            max: min_max.max,
            stats: RasterViewerData {
                stats: band.compute_raster_min_max(false).unwrap(),
                width: band.size().0,
                height: band.size().1,
                no_data_value: band.no_data_value().unwrap_or(NAN),
            },
            data: RawRasterData::new(band, index),
            hist,
            index,
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
            no_data_value,
        }: &Self::Props,
        ImagePoint { x, y }: &Self::State,
    ) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: format!("No data value: {}", no_data_value),
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
                    text: format!("min: {}, max: {}", stats.min, stats.max,),
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
    type Message = (RasterGraphSettings, RasterIndex);
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
                            settings.clone(),
                            props.index,
                        ))
                    }),
                }),
            ),
            (
                54,
                VNode::Button(VButton {
                    text: "Calculate slope".to_string(),
                    click: Some(|data, _| {
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            Action::SlopeRaster(data.index),
                        ))
                    }),
                }),
            ),
        ]
    }

    fn on_message(
        (settings, index): &Self::Message,
        props: &Self::Props,
        state: &mut Self::State,
    ) -> bool {
        if *index == props.index {
            *state = settings.clone();
        }
        false
    }
}

fn render_histagram_row(
    index: usize,
    props: &(Vec<f64>, RasterIndex),
    _: &HistogramSettings,
) -> Vec<VNode<HistComponent>> {
    vec![VNode::Label(VLabel {
        text: props.0[index].to_string(),
    })]
}

#[derive(Clone, PartialEq)]
pub struct HistComponent;
impl Component for HistComponent {
    type Props = (Vec<f64>, RasterIndex);
    type State = HistogramSettings;
    type Message = HistogramSettingsWrapper;
    fn render(props: &Self::Props, _state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Button(VButton {
                    text: "Play histogram".to_owned(),
                    click: Some(|(hist, _), settings| {
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            Action::SendAudioGraph(hist.clone(), settings.clone()),
                        ))
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
                    click: Some(|(_, index), settings| {
                        dispatch_action(Action::SendChangeHistogramSettings(
                            settings.clone(),
                            *index,
                        ))
                    }),
                }),
            ),
        ]
    }

    fn on_message(
        HistogramSettingsWrapper {
            settings,
            index: target,
        }: &Self::Message,
        (_, this_index): &Self::Props,
        state: &mut Self::State,
    ) -> bool {
        if target == this_index {
            *state = settings.clone()
        }
        // Only changes audio stuff, the actual ui stays the same, for now
        false
    }
}

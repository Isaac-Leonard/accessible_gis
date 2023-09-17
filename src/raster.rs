use crate::app::BasicApp;
use crate::events::{dispatch_action, dispatch_click, Action, Click, MessageHandler};
use crate::graph::{generate_image_histogram, HistogramSettings};
use crate::layout::{fill_safe_area, top_to_bottom};
use crate::list_view::{ConfigurableRow, MyListView};
use cacao::layout::{Layout, LayoutConstraint};
use cacao::listview::ListView;
use cacao_framework::{Component, ComponentType, Discripter, Renderable};
use gdal::raster::GdalDataType;

use cacao::button::Button;
use cacao::text::Label;
use cacao::view::{View, ViewDelegate};
use gdal::raster::{RasterBand, StatisticsAll};
use ndarray::Array2;

use std::cell::RefCell;

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

pub struct RasterLayerView {
    pub content: View,
    label: Label,
    position: usize,
    play_pause_btn: Button,
    change_hist_settings_btn: Button,
    playing: bool,
    hist: Option<Vec<f64>>,
    hist_table: Option<ListView<MyListView<HistogramViewRow>>>,
    data_type_name: String,
    hist_settings: RefCell<HistogramSettings>,
    size: (usize, usize),
    raw_data: Option<Array2<u32>>,
    play_rasta_graph_btn: Button,
    stats: View<Component<RasterViewerData, ImagePoint, StatsComponent, BasicApp>>,
}

impl MessageHandler<Action> for RasterLayerView {
    fn on_message(&self, message: &Action) {
        match message {
            Action::UpdateHistogramSettings(position, settings) => {
                if self.position == *position {
                    let mut settings_ptr = self.hist_settings.borrow_mut();
                    *settings_ptr = settings.clone();
                }
            }
            _ => {}
        }
    }
}

impl MessageHandler<usize> for RasterLayerView {
    fn on_message(&self, message: &usize) {
        if let Some(delegate) = &self.stats.delegate {
            delegate.on_message(message);
        }
    }
}
impl MessageHandler<Click> for RasterLayerView {
    fn on_message(&self, message: &Click) {
        match message {
			Click::OpenChangeHistogramSettings(position)=>if *position==self.position{dispatch_action(Action::SendChangeHistogramSettings(self.position,self.hist_settings.borrow().clone() ))}
            Click::PlayHistogramGraph(position) => {
                if let Some(hist)=&self.hist && *position == self.position {
                    dispatch_action(Action::SendAudioGraph(
                        hist.clone(),
                        self.hist_settings.borrow().clone(),
                    ))
                }
            }
            _ => {}
        }
    }
}

impl RasterLayerView {
    pub fn new(band: &RasterBand, position: usize) -> Self {
        let band_type = band.band_type();
        let hist = match band_type {
            GdalDataType::UInt8 => Some(generate_image_histogram(
                band.read_band_as::<u8>().unwrap().data,
            )),
            _ => None,
        };
        let data: Option<Array2<u32>> = match band_type {
            GdalDataType::UInt8 => Some(
                band.read_as_array::<u8>((0, 0), band.size(), band.size(), None)
                    .unwrap()
                    .mapv_into_any(|x| x as u32),
            ),
            GdalDataType::UInt16 => Some(
                band.read_as_array::<u16>((0, 0), band.size(), band.size(), None)
                    .unwrap()
                    .mapv_into_any(|x| x as u32),
            ),
            GdalDataType::UInt32 => Some(
                band.read_as_array::<u32>((0, 0), band.size(), band.size(), None)
                    .unwrap(),
            ),
            _ => None,
        };
        RasterLayerView {
            data_type_name: band_type.name(),
            content: View::new(),
            label: Label::default(),
            position,
            play_pause_btn: Button::new("play"),
            change_hist_settings_btn: Button::new("Change settings for histogram"),
            playing: false,

            hist: hist.clone(),
            hist_table: hist.map(|hist| ListView::with(MyListView::new(hist))),
            hist_settings: RefCell::new(HistogramSettings::default()),
            size: band.size(),
            raw_data: data,
            play_rasta_graph_btn: Button::new("Play rasta graph"),
            stats: View::with(Component::new(RasterViewerData {
                stats: band.get_statistics(true, true).unwrap().unwrap(),
                width: band.size().0,
                height: band.size().1,
            })),
        }
    }
}

impl ViewDelegate for RasterLayerView {
    const NAME: &'static str = "RasterView";

    fn did_load(&mut self, view: View) {
        let position = self.position;

        self.label
            .set_text(format!("Rasta file with {} data", self.data_type_name));
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);
        let hist = self.hist.clone();
        if let Some(_hist) = &hist {
            self.play_pause_btn
                .set_action(move |_| dispatch_click(Click::PlayHistogramGraph(position)));
        }

        self.content.add_subview(&self.play_pause_btn);
        self.change_hist_settings_btn
            .set_action(move |_| dispatch_click(Click::OpenChangeHistogramSettings(position)));
        self.content.add_subview(&self.change_hist_settings_btn);
        if let Some(hist_table) = &self.hist_table {
            self.content.add_subview(hist_table);
        }
        let size = self.size;
        if let Some(data) = self.raw_data.clone() {
            self.play_rasta_graph_btn.set_action(move |_| {
                dispatch_action(Action::PlayRastaGraph(size, data.clone()));
            });
        }
        self.content.add_subview(&self.play_rasta_graph_btn);

        self.content.add_subview(&self.stats);
        view.add_subview(&self.content);
        let references: Vec<&dyn Layout> = if let Some(hist_table) = &self.hist_table {
            vec![
                &self.label,
                &self.play_pause_btn,
                &self.change_hist_settings_btn,
                hist_table,
                &self.stats,
            ]
        } else {
            vec![
                &self.label,
                &self.play_pause_btn,
                &self.change_hist_settings_btn,
                &self.play_rasta_graph_btn,
                &self.stats,
            ]
        };
        let inner_constraints = top_to_bottom(references, &self.content, 16.0);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        LayoutConstraint::activate(
            &fill_safe_area(&self.content, &view.safe_layout_guide)
                .into_iter()
                .chain(inner_constraints)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct ImagePoint {
    x: usize,
    y: usize,
}

pub struct StatsComponent;

impl Renderable for StatsComponent {
    type Props = RasterViewerData;
    type State = ImagePoint;
    fn render(
        RasterViewerData {
            width,
            height,
            stats,
        }: &Self::Props,
        ImagePoint { x, y }: &Self::State,
    ) -> Vec<Discripter<Self::Props, Self::State>> {
        vec![
            Discripter {
                kind: ComponentType::Label,
                text: stats.mean.to_string(),
            },
            Discripter {
                kind: ComponentType::Label,
                text: format!("x: {x}, y: {y}, width: {width}, height: {height}",),
            },
            Discripter {
                kind: ComponentType::Label,
                text: format!("min: {}, max: {}", stats.min, stats.max),
            },
            Discripter {
                kind: ComponentType::Button(Some(|data, point| point.y += data.height)),
                text: "Move North".to_string(),
            },
            Discripter {
                kind: ComponentType::Button(Some(|data, point| {
                    point.y = point.y.saturating_sub(data.height)
                })),
                text: "Move south".to_string(),
            },
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RasterViewerrMessage {
    MoveNorth,
    MoveEast,
    MoveSouth,
    MoveWest,
    DoubleHeight,
    DoubleWidth,
    HalveHeight,
    HalveWidth,
}

#[derive(Default)]
pub struct HistogramViewRow {
    value: Label,
}

impl ConfigurableRow for HistogramViewRow {
    type Data = f64;
    fn configure_with(&mut self, data: &Self::Data) {
        self.value.set_text(data.to_string());
    }
}

impl ViewDelegate for HistogramViewRow {
    const NAME: &'static str = "HistogramViewRow";
    fn did_load(&mut self, view: View) {
        view.add_subview(&self.value);
        LayoutConstraint::activate(&[
            self.value.top.constraint_equal_to(&view.top).offset(8.),
            self.value
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.value
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
            self.value
                .bottom
                .constraint_equal_to(&view.bottom)
                .offset(-8.),
        ]);
    }
}

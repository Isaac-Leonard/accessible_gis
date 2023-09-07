use crate::events::{dispatch_ui, Message};
use crate::graph::freq_counts;
use cacao::layout::{Layout, LayoutConstraint};
use gdal::raster::GdalDataType;

use cacao::text::Label;
use cacao::view::{View, ViewDelegate};
use cacao::{button::Button, view};
use gdal::raster::{RasterBand, StatisticsAll};

use std::cell::RefCell;

use std::rc::Rc;

pub struct RasterView {
    layers: RasterLayerView,
}

pub struct RasterViewerData {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    stats: StatisticsAll,
}

pub struct RasterLayerView {
    pub content: view::View,
    label: Label,
    position: usize,
    play_pause_btn: Button,
    move_west_btn: Button,
    move_east_btn: Button,
    move_north_btn: Button,
    move_south_btn: Button,
    double_width_btn: Button,
    double_height_btn: Button,
    halve_width_btn: Button,
    halve_height_btn: Button,
    playing: bool,
    cell_value_label: Label,
    positional_information_label: Label,
    stats_label: Label,
    data: Rc<RefCell<RasterViewerData>>,
    hist: Vec<f64>,
}

impl RasterLayerView {
    pub fn on_message(&self, message: &Message) {
        match message {
            Message::RasterViewerAction(action) => {
                // Must keep the mutable borrow of data in its own block so its released before calling update_value
                {
                    let mut data = self.data.borrow_mut();
                    match action {
                        RasterViewerrMessage::MoveNorth => data.y += data.height,
                        RasterViewerrMessage::MoveEast => data.x += data.width,
                        RasterViewerrMessage::MoveSouth => {
                            data.y = data.y.checked_sub(data.height).unwrap_or(0)
                        }
                        RasterViewerrMessage::MoveWest => {
                            data.x = data.x.checked_sub(data.width).unwrap_or(0)
                        }

                        RasterViewerrMessage::HalveWidth => {
                            data.width = data.width / 2;
                        }
                        RasterViewerrMessage::DoubleWidth => {
                            data.width = data.width * 2;
                        }
                        RasterViewerrMessage::HalveHeight => {
                            data.height = data.height / 2;
                        }
                        RasterViewerrMessage::DoubleHeight => {
                            data.height = data.height * 2;
                        }
                    };
                    if data.width == 0 {
                        data.width = 1
                    }
                    if data.height == 0 {
                        data.height = 1
                    }
                }
                self.update_value();
            }
            _ => {}
        }
    }
}

impl RasterLayerView {
    pub fn new(band: &RasterBand, position: usize) -> Self {
        let band_type = band.band_type();
        RasterLayerView {
            content: View::new(),
            label: Label::default(),
            position,
            play_pause_btn: Button::new("play"),
            playing: false,
            move_east_btn: Button::new("East"),
            move_west_btn: Button::new("West"),
            move_north_btn: Button::new("North"),
            move_south_btn: Button::new("South"),
            double_width_btn: Button::new("Double width"),
            halve_width_btn: Button::new("Half width"),
            double_height_btn: Button::new("Double height"),
            halve_height_btn: Button::new("Halve height"),
            data: Rc::new(RefCell::new(RasterViewerData {
                stats: band.get_statistics(true, true).unwrap().unwrap(),
                x: 0,
                y: 0,
                width: band.size().0,
                height: band.size().1,
            })),
            positional_information_label: Label::new(),
            cell_value_label: Label::new(),
            stats_label: Label::new(),
            hist: match band_type {
                GdalDataType::Int8 => freq_counts(band.read_band_as::<i8>().unwrap().data),
                GdalDataType::UInt8 => freq_counts(band.read_band_as::<u8>().unwrap().data),
                GdalDataType::Int16 => freq_counts(band.read_band_as::<i16>().unwrap().data),
                GdalDataType::UInt16 => freq_counts(band.read_band_as::<u16>().unwrap().data),
                GdalDataType::Int32 => freq_counts(band.read_band_as::<i32>().unwrap().data),
                GdalDataType::UInt32 => freq_counts(band.read_band_as::<u32>().unwrap().data),
                GdalDataType::Int64 => freq_counts(band.read_band_as::<i64>().unwrap().data),
                GdalDataType::UInt64 => freq_counts(band.read_band_as::<u64>().unwrap().data),
                GdalDataType::Float32 => freq_counts(band.read_band_as::<f32>().unwrap().data),
                GdalDataType::Float64 => freq_counts(band.read_band_as::<f64>().unwrap().data),
                GdalDataType::Unknown => panic!("Unknown datatype for rasta"),
            },
        }
    }
}

impl ViewDelegate for RasterLayerView {
    const NAME: &'static str = "RasterView";

    fn did_load(&mut self, view: View) {
        macro_rules! connect_button {
            ($btn:expr, $action:expr) => {{
                $btn.set_action(|| dispatch_ui(Message::RasterViewerAction($action)));
                self.content.add_subview(&$btn);
            }};
        }

        self.label.set_text("Raster file");
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);
        let hist = self.hist.clone();
        self.play_pause_btn
            .set_action(move || dispatch_ui(Message::PlayAudioGraph(hist.clone())));
        self.content.add_subview(&self.play_pause_btn);
        connect_button!(self.move_north_btn, RasterViewerrMessage::MoveNorth);
        connect_button!(self.move_east_btn, RasterViewerrMessage::MoveEast);
        connect_button!(self.move_south_btn, RasterViewerrMessage::MoveSouth);
        connect_button!(self.move_west_btn, RasterViewerrMessage::MoveWest);
        connect_button!(self.double_height_btn, RasterViewerrMessage::DoubleHeight);
        connect_button!(self.halve_height_btn, RasterViewerrMessage::HalveHeight);
        connect_button!(self.double_width_btn, RasterViewerrMessage::DoubleWidth);
        connect_button!(self.halve_width_btn, RasterViewerrMessage::HalveWidth);

        self.content.add_subview(&self.cell_value_label);
        self.content.add_subview(&self.positional_information_label);
        self.content.add_subview(&self.stats_label);
        self.update_value();
        view.add_subview(&self.content);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top)
                .offset(self.position as f64 * 50.),
            self.content
                .leading
                .constraint_equal_to(&view.safe_layout_guide.leading),
            self.content
                .trailing
                .constraint_equal_to(&view.safe_layout_guide.trailing),
            self.content
                .bottom
                .constraint_equal_to(&view.safe_layout_guide.bottom),
        ])
    }
}

impl RasterLayerView {
    fn update_value(&self) {
        let data = self.data.borrow();
        self.cell_value_label.set_text(data.stats.mean.to_string());
        self.positional_information_label.set_text(format!(
            "x: {}, y: {}, width: {}, height: {}",
            data.x, data.y, data.width, data.height
        ));
        self.stats_label
            .set_text(format!("min: {}, max: {}", data.stats.min, data.stats.max));
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

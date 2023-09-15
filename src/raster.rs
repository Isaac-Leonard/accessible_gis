use crate::events::{dispatch_ui, Message, MessageHandler};
use crate::graph::{generate_image_histogram, HistogramSettings};
use crate::layout::{fill_safe_area, top_to_bottom, HasLayout};
use crate::list_view::{ConfigurableRow, MyListView};
use cacao::layout::{Layout, LayoutConstraint};
use cacao::listview::ListView;
use gdal::raster::GdalDataType;

use cacao::button::Button;
use cacao::text::Label;
use cacao::view::{View, ViewDelegate};
use gdal::raster::{RasterBand, StatisticsAll};
use ndarray::Array2;

use std::cell::RefCell;

use std::rc::Rc;

pub struct RasterViewerData {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    stats: StatisticsAll,
}

pub struct RasterLayerView {
    pub content: View,
    label: Label,
    position: usize,
    play_pause_btn: Button,
    change_hist_settings_btn: Button,
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
    hist: Option<Vec<f64>>,
    hist_table: Option<ListView<MyListView<HistogramViewRow>>>,
    data_type_name: String,
    hist_settings: Rc<RefCell<HistogramSettings>>,
    size: (usize, usize),
    raw_data: Option<Array2<u32>>,
    play_rasta_graph_btn: Button,
}

impl MessageHandler for RasterLayerView {
    type Message = Message;
    fn on_message(&self, message: &Self::Message) {
        match message {
            Message::RasterViewerAction(action) => {
                // Must keep the mutable borrow of data in its own block so its released before calling update_value
                {
                    let mut data = self.data.borrow_mut();
                    match action {
                        RasterViewerrMessage::MoveNorth => data.y += data.height,
                        RasterViewerrMessage::MoveEast => data.x += data.width,
                        RasterViewerrMessage::MoveSouth => {
                            data.y = data.y.saturating_sub(data.height)
                        }
                        RasterViewerrMessage::MoveWest => {
                            data.x = data.x.saturating_sub(data.width)
                        }

                        RasterViewerrMessage::HalveWidth => {
                            data.width /= 2;
                        }
                        RasterViewerrMessage::DoubleWidth => {
                            data.width *= 2;
                        }
                        RasterViewerrMessage::HalveHeight => {
                            data.height /= 2;
                        }
                        RasterViewerrMessage::DoubleHeight => {
                            data.height *= 2;
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
			Message::OpenChangeHistogramSettings(position)=>if *position==self.position{dispatch_ui(Message::SendChangeHistogramSettings(self.position,self.hist_settings.borrow().clone() ))}
            Message::UpdateHistogramSettings(position, settings) => {
                if self.position == *position {
                    let mut settings_ptr = self.hist_settings.borrow_mut();
                    *settings_ptr = settings.clone();
                }
            }
            Message::PlayHistogramGraph(position) => {
                if let Some(hist)=&self.hist && *position == self.position {
                    dispatch_ui(Message::SendAudioGraph(
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
            hist: hist.clone(),
            hist_table: hist.map(|hist| ListView::with(MyListView::new(hist))),
            hist_settings: Rc::new(RefCell::new(HistogramSettings::default())),
            size: band.size(),
            raw_data: data,
            play_rasta_graph_btn: Button::new("Play rasta graph"),
        }
    }
}

impl ViewDelegate for RasterLayerView {
    const NAME: &'static str = "RasterView";

    fn did_load(&mut self, view: View) {
        let position = self.position;
        macro_rules! connect_button {
            ($btn:expr, $action:expr) => {{
                $btn.set_action(|_| dispatch_ui(Message::RasterViewerAction($action)));
                self.content.add_subview(&$btn);
            }};
        }

        self.label
            .set_text(format!("Rasta file with {} data", self.data_type_name));
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);
        let hist = self.hist.clone();
        if let Some(_hist) = &hist {
            self.play_pause_btn.set_action(move |_| {
                dispatch_ui(Message::PlayHistogramGraph(position));
            });
        }

        self.content.add_subview(&self.play_pause_btn);
        self.change_hist_settings_btn
            .set_action(move |_| dispatch_ui(Message::OpenChangeHistogramSettings(position)));
        self.content.add_subview(&self.change_hist_settings_btn);
        if let Some(hist_table) = &self.hist_table {
            self.content.add_subview(hist_table);
        }
        let size = self.size;
        if let Some(data) = self.raw_data.clone() {
            self.play_rasta_graph_btn.set_action(move |_| {
                dispatch_ui(Message::PlayRastaGraph(size, data.clone()));
            });
        }
        self.content.add_subview(&self.play_rasta_graph_btn);

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
        let references: Vec<&dyn HasLayout> = if let Some(hist_table) = &self.hist_table {
            vec![
                &self.label,
                &self.play_pause_btn,
                &self.change_hist_settings_btn,
                hist_table,
                &self.stats_label,
                &self.positional_information_label,
                &self.cell_value_label,
                &self.move_north_btn,
                &self.move_south_btn,
                &self.move_west_btn,
                &self.move_east_btn,
                &self.double_height_btn,
                &self.halve_height_btn,
                &self.double_width_btn,
                &self.halve_width_btn,
            ]
        } else {
            vec![
                &self.label,
                &self.play_pause_btn,
                &self.change_hist_settings_btn,
                &self.play_rasta_graph_btn,
                &self.stats_label,
                &self.positional_information_label,
                &self.cell_value_label,
                &self.move_north_btn,
                &self.move_south_btn,
                &self.move_west_btn,
                &self.move_east_btn,
                &self.double_height_btn,
                &self.halve_height_btn,
                &self.double_width_btn,
                &self.halve_width_btn,
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

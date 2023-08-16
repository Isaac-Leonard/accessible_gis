use crate::app::dispatch_ui;
use crate::audio::{get_audio, AudioMessage};
use cacao::filesystem::FileSelectPanel;
use cacao::foundation::NSURL;
use cacao::layout::{Layout, LayoutConstraint};
use cacao::listview::{ListView, ListViewDelegate};
use cacao::notification_center::Dispatcher;
use cacao::text::Label;
use cacao::view::{View, ViewDelegate};
use cacao::{button::Button, view};
use gdal::raster::{reproject, RasterBand, StatisticsAll};
use gdal::vector::{Feature, FieldValue, Geometry, LayerAccess};
use gdal::Dataset;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::Sender;

pub struct MainView {
    content: view::View,
    button: Button,
    dataset_view: Rc<RefCell<Option<View<DatasetView>>>>,
}
impl MainView {
    pub fn new() -> Self {
        Self {
            content: View::new(),
            button: Button::new("Select file"),
            dataset_view: Rc::new(RefCell::new(None)),
        }
    }
}

impl Dispatcher for MainView {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        match message {
            Message::ToggleAudio | Message::RasterViewerAction(_) => {
                let dataset_view = self.dataset_view.borrow_mut();
                dataset_view
                    .as_ref()
                    .and_then(|x| x.delegate.as_ref())
                    .inspect(|x| x.on_ui_message(message));
            }
            Message::ClickedSelectFile => FileSelectPanel::new().show(file_selection_handler),
            Message::GotFile(path) => {
                let dataset = Dataset::open(path).expect("Could'nt read file");
                let mut dataset_view = self.dataset_view.borrow_mut();
                let sub_view = View::with(DatasetView::new(dataset));
                self.content.add_subview(&sub_view);
                *dataset_view = Some(sub_view);
            }
            Message::InvalidFile(_) => {}
        }
    }
}

pub struct DatasetView {
    content: view::View,
    sub_views: Rc<RefCell<Vec<View<LayerView>>>>,
    audio: Sender<AudioMessage>,
    dataset: Dataset,
    spatial_reference_label: Label,
}

impl DatasetView {
    fn new(dataset: Dataset) -> Self {
        eprintln!("Here");
        let audio = get_audio();
        let view = Self {
            content: View::new(),
            dataset,
            audio,
            sub_views: Rc::new(RefCell::new(Vec::new())),
            spatial_reference_label: Label::new(),
        };
        let mut sub_views = Vec::new();
        let layers = view.dataset.layers();
        let mut last_position = 0;
        for mut layer in layers {
            for feature in layer.features() {
                let vector_view = View::with(FeatureView::new(feature, last_position));
                vector_view.set_background_color(cacao::color::Color::SystemRed);
                sub_views.push(View::with(LayerView::Vector(vector_view)));
                last_position += 1;
            }
        }
        // TODO: Lets try replace with a proper iterator
        for i in 1..=view.dataset.raster_count() {
            let band = view.dataset.rasterband(i).unwrap();
            let raster_view = View::with(RasterView::new(band, i as usize + last_position));
            raster_view.set_background_color(cacao::color::Color::SystemRed);
            sub_views.push(View::with(LayerView::Raster(raster_view)));
        }
        view.sub_views.borrow_mut().append(&mut sub_views);
        view
    }
}

pub enum LayerView {
    Vector(View<FeatureView>),
    Raster(View<RasterView>),
}

impl ViewDelegate for LayerView {
    const NAME: &'static str = "LayerView";
    fn did_load(&mut self, view: View) {
        match self {
            Self::Raster(raster) => view.add_subview(raster),
            Self::Vector(vector) => view.add_subview(vector),
        }
        LayoutConstraint::activate(&[
            view.height.constraint_equal_to_constant(300.),
            view.width.constraint_equal_to_constant(600.),
        ])
    }
}

impl LayerView {
    fn on_message(&self, message: Message) {
        match self {
            Self::Vector(_vector_view) => {}
            Self::Raster(raster_view) => {
                if let Some(ref view) = raster_view.delegate {
                    view.as_ref().on_message(message)
                }
            }
        }
    }
}

impl ViewDelegate for MainView {
    const NAME: &'static str = "SafeAreaView";

    fn did_load(&mut self, view: View) {
        self.button
            .set_action(|| dispatch_ui(Message::ClickedSelectFile));
        self.button.set_key_equivalent("c");
        self.content.add_subview(&self.button);
        view.add_subview(&self.content);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        cacao::layout::LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top),
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

impl ViewDelegate for DatasetView {
    const NAME: &'static str = "DataSetView";

    fn did_load(&mut self, view: View) {
        for sub_view in self.sub_views.borrow().iter() {
            self.content.add_subview(sub_view)
        }
        self.spatial_reference_label.set_text(
            self.dataset
                .spatial_ref()
                .unwrap_or_else(|_| self.dataset.layer(0).unwrap().spatial_ref().unwrap())
                .to_pretty_wkt()
                .unwrap(),
        );
        self.content.add_subview(&self.spatial_reference_label);
        view.add_subview(&self.content);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top),
            self.content
                .leading
                .constraint_equal_to(&view.safe_layout_guide.leading),
            self.content
                .trailing
                .constraint_equal_to(&view.safe_layout_guide.trailing),
            self.content
                .bottom
                .constraint_equal_to(&view.safe_layout_guide.bottom),
            self.spatial_reference_label
                .top
                .constraint_equal_to(&self.content.top),
            self.spatial_reference_label
                .leading
                .constraint_equal_to(&self.content.leading),
            self.spatial_reference_label
                .height
                .constraint_equal_to_constant(40.0),
            self.spatial_reference_label
                .width
                .constraint_equal_to_constant(80.0),
        ])
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
        dispatch_ui(Message::InvalidFile(path));
        return;
    }
    dispatch_ui(Message::GotFile(path))
}

impl Dispatcher for DatasetView {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        match message {
            Message::ToggleAudio => self.audio.send(AudioMessage::PlayPause).unwrap(),
            Message::RasterViewerAction(action) => {
                let views = self.sub_views.borrow();
                views
                    .iter()
                    .filter_map(|v| v.delegate.as_ref())
                    .for_each(|view| view.on_message(Message::RasterViewerAction(action)));
            }
            Message::InvalidFile(_) => {}
            Message::GotFile(_) => {}
            Message::ClickedSelectFile => {}
        }
    }
}

pub struct FeatureView {
    pub content: view::View,
    label: Label,
    attribute_table: ListView<AttributesListView>,
    position: usize,
    projection_label: Label,
    geometry: Geometry,
}

impl FeatureView {
    fn new(feature: Feature, position: usize) -> Self {
        FeatureView {
            content: View::new(),
            label: Label::default(),
            attribute_table: ListView::with(AttributesListView::new(feature.fields().collect())),
            position,
            projection_label: Label::new(),
            geometry: feature.geometry().unwrap().clone(),
        }
    }
}

impl ViewDelegate for FeatureView {
    const NAME: &'static str = "VectorView";

    fn did_load(&mut self, view: View) {
        self.content.add_subview(&self.attribute_table);
        self.label.set_text(self.geometry.geometry_name());
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);

        self.content.add_subview(&self.projection_label);
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

pub struct RasterViewerData {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    stats: StatisticsAll,
}

pub struct RasterView {
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
}

impl RasterView {
    fn on_message(&self, message: Message) {
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

impl RasterView {
    fn new(band: RasterBand, position: usize) -> Self {
        RasterView {
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
        }
    }
}

impl ViewDelegate for RasterView {
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
        self.play_pause_btn
            .set_action(|| dispatch_ui(Message::ToggleAudio));
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

impl RasterView {
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

#[derive(Clone, Debug)]
pub enum Message {
    ClickedSelectFile,
    GotFile(PathBuf),
    InvalidFile(PathBuf),
    ToggleAudio,
    RasterViewerAction(RasterViewerrMessage),
}

#[derive(Default, Debug)]
pub struct AttributeViewRow {
    pub key: Label,
    pub value: Label,
}

type Attribute = (String, Option<FieldValue>);

impl AttributeViewRow {
    /// Called when this view is being presented, and configures itself     pub
    fn configure_with(&mut self, (key, val): &Attribute) {
        self.key.set_text(key);
        self.value.set_text(format!("{:?}", val));
    }
}

impl ViewDelegate for AttributeViewRow {
    const NAME: &'static str = "AttributeViewRow";

    /// Called when the view is first created; handles setup of layout and associated styling that
    /// doesn't change.
    fn did_load(&mut self, view: View) {
        view.add_subview(&self.key);
        view.add_subview(&self.value);

        LayoutConstraint::activate(&[
            self.key.top.constraint_equal_to(&view.top).offset(16.),
            self.key
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.key
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
            self.value
                .top
                .constraint_equal_to(&self.key.bottom)
                .offset(8.),
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
                .offset(-16.),
        ]);
    }
}

/// An identifier for the cell(s) we dequeue.
const ATTRIBUTE_ROW: &str = "AttributeViewRowCell";

/// The list view for attributes
pub struct AttributesListView {
    view: Option<ListView>,
    attributes: Vec<Attribute>,
}

impl AttributesListView {
    fn new(attributes: Vec<Attribute>) -> Self {
        Self {
            view: None,
            attributes,
        }
    }
}

impl ListViewDelegate for AttributesListView {
    const NAME: &'static str = "AttributesListView";

    /// Essential configuration and retaining of a `ListView` handle to do updates later on.
    fn did_load(&mut self, view: ListView) {
        eprintln!("called");
        view.register(ATTRIBUTE_ROW, AttributeViewRow::default);
        view.set_uses_alternating_backgrounds(true);
        view.set_row_height(64.);
        LayoutConstraint::activate(&[
            view.height.constraint_equal_to_constant(50.0),
            view.width.constraint_equal_to_constant(50.0),
        ]);
        self.view = Some(view);
    }

    /// The number of attributes we have.
    fn number_of_items(&self) -> usize {
        self.attributes.len()
    }

    /// For a given row, dequeues a view from the system and passes the appropriate `Transfer` for
    /// configuration.
    fn item_for(&self, row: usize) -> cacao::listview::ListViewRow {
        eprintln!(
            "item ffor called with {:?} and len of {}",
            row,
            self.attributes.len()
        );
        let mut view = self
            .view
            .as_ref()
            .unwrap()
            .dequeue::<AttributeViewRow>(ATTRIBUTE_ROW);

        if let Some(view) = &mut view.delegate {
            let attribute = &self.attributes[row];
            view.configure_with(attribute);
        }

        view.into_row()
    }
}

use cacao::filesystem::FileSelectPanel;
use cacao::foundation::NSURL;
use cacao::layout::{Layout, LayoutConstraint};
use cacao::listview::{ListView, ListViewDelegate};
use cacao::notification_center::Dispatcher;
use cacao::text::Label;
use cacao::view::{View, ViewDelegate};
use cacao::{button, button::Button, view};

use geotiff::TIFF;
use shapefile::dbase::FieldValue;
use shapefile::{read, Shape};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::app::dispatch_ui;
use crate::audio::{get_audio, AudioMessage};
#[derive(Default)]
pub struct ContentView {
    content: view::View,
    button: Option<Button>,
    sub_views: Rc<RefCell<Vec<LayerView>>>,
    audio: Option<Sender<AudioMessage>>,
}

pub enum LayerView {
    Shape(View<ShapeView>),
    Tiff(View<TiffView>),
}

impl LayerView {
    fn on_message(&self, message: Message) {
        match self {
            Self::Shape(_shape_view) => {}
            Self::Tiff(tiff_view) => {
                if let Some(ref view) = tiff_view.delegate {
                    view.as_ref().on_message(message)
                }
            }
        }
    }
}

impl ViewDelegate for ContentView {
    const NAME: &'static str = "SafeAreaView";

    fn did_load(&mut self, view: View) {
        let mut btn = button::Button::new("Select file");
        btn.set_action(|| dispatch_ui(Message::ClickedSelectFile));
        btn.set_key_equivalent("c");
        self.content.add_subview(&btn);
        self.button = Some(btn);
        view.add_subview(&self.content);
        self.audio = Some(get_audio());
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

fn file_selection_handler(paths: Vec<NSURL>) {
    if paths.is_empty() {
        // User canceled
        return;
    }
    let path = paths[0].pathbuf();
    // Simplistic check for shape file
    if path.is_dir() {
        dispatch_ui(Message::InvalidFile(path));
        return;
    }
    let path_as_str = path.to_str().expect("non-Utf8 file path found");
    if path_as_str.ends_with(".shp") {
        dispatch_ui(Message::GotShapeFile(path))
    } else if path_as_str.ends_with(".tif") {
        dispatch_ui(Message::GotTiffFile(path))
    } else {
        dispatch_ui(Message::InvalidFile(path));
    }
}

impl Dispatcher for ContentView {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        match message {
            Message::ToggleAudio => self
                .audio
                .as_ref()
                .unwrap()
                .send(AudioMessage::PlayPause)
                .unwrap(),
            Message::ClickedSelectFile => FileSelectPanel::new().show(file_selection_handler),
            Message::GotShapeFile(path) => {
                let file = read(path).expect("Could'nt read shape file");
                for (shape, record) in file {
                    let shape_view = View::with(ShapeView::new(
                        shape,
                        record.into_iter().collect(),
                        self.sub_views.borrow_mut().len(),
                    ));
                    shape_view.set_background_color(cacao::color::Color::SystemRed);
                    self.content.add_subview(&shape_view);
                    self.sub_views
                        .borrow_mut()
                        .push(LayerView::Shape(shape_view));
                }
            }
            // Basic display for now
            Message::GotTiffFile(path) => {
                let tiff =
                    geotiff::TIFF::open(path).expect("Something went wrong reading the tiff file");
                let tiff_view = View::with(TiffView::new(tiff, self.sub_views.borrow_mut().len()));
                tiff_view.set_background_color(cacao::color::Color::SystemRed);
                self.content.add_subview(&tiff_view);
                self.sub_views.borrow_mut().push(LayerView::Tiff(tiff_view));
            }
            // Don't do anything for now
            Message::InvalidFile(_) => {}
            Message::TiffViewerAction(action) => {
                let views = self.sub_views.borrow();
                views
                    .iter()
                    .for_each(|view| view.on_message(Message::TiffViewerAction(action)))
            }
        }
    }
}

pub struct ShapeView {
    pub content: view::View,
    label: Label,
    shape: Shape,
    attribute_table: ListView<AttributesListView>,
    position: usize,
}

impl ShapeView {
    fn new(shape: Shape, record: Vec<Attribute>, position: usize) -> Self {
        ShapeView {
            content: View::new(),
            label: Label::default(),
            shape,
            attribute_table: ListView::with(AttributesListView::new(record)),
            position,
        }
    }
}

impl ViewDelegate for ShapeView {
    const NAME: &'static str = "ShapeView";

    fn did_load(&mut self, view: View) {
        self.content.add_subview(&self.attribute_table);
        self.label.set_text(format!("{:?}", self.shape.shapetype()));
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);
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

pub struct TiffViewerData {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

pub struct TiffView {
    pub content: view::View,
    label: Label,
    tiff: Box<TIFF>,
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
    data: Rc<RefCell<TiffViewerData>>,
}

impl TiffView {
    fn on_message(&self, message: Message) {
        match message {
            Message::TiffViewerAction(action) => {
                match action {
                    TiffViewerrMessage::HalveWidth => {
                        eprintln!("Halving width");
                        let mut data = self.data.borrow_mut();
                        data.width = data.width / 2;
                    }
                    _ => {}
                };
                self.update_value();
            }
            _ => {}
        }
    }
}

impl TiffView {
    fn new(tiff: Box<TIFF>, position: usize) -> Self {
        TiffView {
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
            data: Rc::new(RefCell::new(TiffViewerData {
                x: 0,
                y: 0,
                width: tiff.image_data[0].len(),
                height: tiff.image_data.len(),
            })),
            positional_information_label: Label::new(),
            cell_value_label: Label::new(),
            tiff,
        }
    }
}

impl ViewDelegate for TiffView {
    const NAME: &'static str = "TiffView";

    fn did_load(&mut self, view: View) {
        macro_rules! connect_button {
            ($btn:expr, $action:expr) => {{
                $btn.set_action(|| dispatch_ui(Message::TiffViewerAction($action)));
                self.content.add_subview(&$btn);
            }};
        }

        self.label.set_text("Tiff file");
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);
        self.play_pause_btn
            .set_action(|| dispatch_ui(Message::ToggleAudio));
        self.content.add_subview(&self.play_pause_btn);
        connect_button!(self.move_north_btn, TiffViewerrMessage::MoveNorth);
        connect_button!(self.move_east_btn, TiffViewerrMessage::MoveEast);
        connect_button!(self.move_south_btn, TiffViewerrMessage::MoveSouth);
        connect_button!(self.move_west_btn, TiffViewerrMessage::MoveWest);
        connect_button!(self.double_height_btn, TiffViewerrMessage::DoubleHeight);
        connect_button!(self.halve_height_btn, TiffViewerrMessage::HalveHeight);
        connect_button!(self.double_width_btn, TiffViewerrMessage::DoubleWidth);
        connect_button!(self.halve_width_btn, TiffViewerrMessage::HalveWidth);

        self.content.add_subview(&self.cell_value_label);
        self.content.add_subview(&self.move_north_btn);
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

impl TiffView {
    fn update_value(&self) {
        let data = self.data.borrow();
        self.cell_value_label.set_text(
            calc_average_value(self.tiff.as_ref(), data.x, data.y, data.width, data.height)
                .to_string(),
        );
    }
}
#[derive(Debug, Clone, Copy)]
pub enum TiffViewerrMessage {
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
    GotShapeFile(PathBuf),
    GotTiffFile(PathBuf),
    InvalidFile(PathBuf),
    ToggleAudio,
    TiffViewerAction(TiffViewerrMessage),
}

#[derive(Default, Debug)]
pub struct AttributeViewRow {
    pub key: Label,
    pub value: Label,
}

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
type Attribute = (String, FieldValue);

/// The list view for attributes
#[derive(Debug)]
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

fn calc_average_value(tiff: &TIFF, x: usize, y: usize, width: usize, height: usize) -> usize {
    let mut val = 0;
    for j in x..(x + width) {
        for i in y..(y + height) {
            val += tiff.image_data[i][j][0];
        }
    }
    val / width / height
}

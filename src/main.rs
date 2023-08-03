use cacao::appkit::window::Window;
use cacao::appkit::{App, AppDelegate};
use cacao::button::Button;
use cacao::filesystem::FileSelectPanel;
use cacao::layout::{Layout, LayoutConstraint};
use cacao::listview::{ListView, ListViewDelegate, RowAnimation};
use cacao::notification_center::Dispatcher;
use cacao::text::Label;
use cacao::view::{View, ViewController, ViewDelegate};
use cacao::{button, view};
use shapefile::dbase::FieldValue;
use shapefile::{read, Shape};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct BasicApp {
    window: Window,
    content_view: ViewController<ContentView>,
}

impl Default for BasicApp {
    fn default() -> Self {
        Self {
            window: Window::default(),
            content_view: ViewController::new(ContentView::default()),
        }
    }
}

impl Dispatcher for BasicApp {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        if let Some(delegate) = &self.content_view.view.delegate {
            delegate.on_ui_message(message);
        }
    }
}

impl AppDelegate for BasicApp {
    fn did_finish_launching(&self) {
        App::activate();
        self.window.set_minimum_content_size(400., 400.);
        self.window.set_title("Hello World!");
        self.window.set_movable_by_background(true);
        self.window.set_content_view_controller(&self.content_view);
        self.window.show();
        //        FileSelectPanel::new().show(|_| {});
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        true
    }
}

#[derive(Default)]
struct ContentView {
    content: view::View,
    button: Option<Button>,
    sub_views: Rc<RefCell<Vec<View<ShapeView>>>>,
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

impl Dispatcher for ContentView {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        match message {
            Message::ClickedSelectFile => {
                FileSelectPanel::new().show(|path| dispatch_ui(Message::GotFile(path[0].pathbuf())))
            }
            Message::GotFile(path) => {
                eprint!("Got file");
                let file = read(path).expect("Could'nt read shape file");
                for (shape, record) in file {
                    let shape_view =
                        View::with(ShapeView::new(shape, record.into_iter().collect()));
                    shape_view.set_background_color(cacao::color::Color::SystemRed);
                    LayoutConstraint::activate(&[
                        shape_view.width.constraint_equal_to_constant(100.0),
                        shape_view.height.constraint_equal_to_constant(100.0),
                    ]);
                    self.content.add_subview(&shape_view);

                    self.sub_views.borrow_mut().push(shape_view);
                }
            }
        }
    }
}

struct ShapeView {
    pub content: view::View,
    label: Label,
    shape: Shape,
    attribute_table: ListView<AttributesListView>,
}

impl ShapeView {
    fn new(shape: Shape, record: Vec<Attribute>) -> Self {
        ShapeView {
            content: View::new(),
            label: Label::default(),
            shape,
            attribute_table: ListView::with(AttributesListView::new(record)),
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

/// Dispatch a message on a background thread.
fn dispatch_ui(message: Message) {
    println!("Dispatching UI message: {:?}", message);
    App::<BasicApp, Message>::dispatch_main(message);
}

#[derive(Clone, Debug)]
enum Message {
    ClickedSelectFile,
    GotFile(PathBuf),
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
        self.view = Some(view);
        self.view
            .as_ref()
            .unwrap()
            .perform_batch_updates(|list_view| {
                let x = (0..self.attributes.len()).rev().collect::<Vec<_>>();
                eprintln!("{:?}", x);
                list_view.insert_rows(&x, RowAnimation::SlideUp)
            });
    }

    /// The number of todos we currently have.
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
            let attribute = &self.attributes[row - 25];
            view.configure_with(attribute);
        }

        view.into_row()
    }
}

fn main() {
    App::new("com.hello.world", BasicApp::default()).run();
}

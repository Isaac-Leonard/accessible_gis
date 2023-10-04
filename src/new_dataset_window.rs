use cacao::appkit::window::{Window, WindowDelegate};
use cacao::view::ViewController;
use cacao_framework::{Component, ComponentWrapper, Message, VButton, VLabel, VNode};
use gdal::raster::GdalDataType;
use gdal::DriverManager;
use std::path::PathBuf;

use gdal::Dataset;

use crate::app::BasicApp;
use crate::events::{dispatch_action, Action, MessageHandler};

pub fn save(name: PathBuf, driver: &str) -> Result<(), ()> {
    let driver = DriverManager::get_driver_by_name(driver).map_err(|x| ())?;
    let dataset = driver.create("in-memory", 64, 64, 1);
    Err(())
}

#[derive(PartialEq, Clone)]
pub enum State {
    Valid(ValidState),
    Invalid(String),
}
impl Default for State {
    fn default() -> Self {
        State::Invalid("Not implemented fully yet".to_owned())
    }
}

#[derive(PartialEq, Clone)]
pub struct ValidState {
    available_drivers: Vec<String>,
}

#[derive(PartialEq, Clone)]
pub struct RasterOptions {
    pub x: usize,
    pub y: usize,
    pub missed_requirements: Vec<String>,
}

#[derive(PartialEq, Clone)]
pub struct VectorOptions;

#[derive(PartialEq, Clone)]
pub enum Options {
    Vector(VectorOptions),
    Raster(RasterOptions),
}

#[derive(PartialEq, Clone)]
pub struct NewDatasetComponent;

impl Component for NewDatasetComponent {
    type Props = ();
    type State = State;
    fn render(_: &Self::Props, state: &Self::State) -> Vec<(usize, cacao_framework::VNode<Self>)> {
        match state {
            State::Invalid(ref msg) => vec![
                (
                    0,
                    VNode::Label(VLabel {
                        text: msg.to_owned(),
                    }),
                ),
                (
                    1,
                    VNode::Button(VButton {
                        text: "Retry".to_owned(),
                        click: Some(|_, state| *state = State::default()),
                    }),
                ),
            ],
            State::Valid(state) => vec![],
        }
    }
}

pub struct NewDatasetWindow {
    pub content: ViewController<ComponentWrapper<NewDatasetComponent, BasicApp>>,
}

impl NewDatasetWindow {
    pub fn new() -> Self {
        let content = ViewController::new(ComponentWrapper::new(()));
        Self { content }
    }
}

impl MessageHandler<Message> for NewDatasetWindow {
    fn on_message(&self, message: &Message) {
        if let Some(delegate) = &self.content.view.delegate {
            delegate.on_message(message)
        };
    }
}

impl WindowDelegate for NewDatasetWindow {
    const NAME: &'static str = "ChangeHistogramSettingsWindow";

    fn did_load(&mut self, window: Window) {
        window.set_autosave_name("NewDatasetWindow");
        window.set_minimum_content_size(600, 400);
        window.set_title("Create new dataset");
        window.set_content_view_controller(&self.content);
    }

    fn cancel(&self) {
        dispatch_action(Action::CloseNewDatasetWindow);
    }
}

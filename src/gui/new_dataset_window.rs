use cacao::appkit::window::{Window, WindowDelegate};
use cacao::appkit::App;
use cacao::view::ViewController;
use cacao_framework::{
    Component, ComponentWrapper, Message, VButton, VComponent, VLabel, VNode, VSelect,
};
use gdal::raster::GdalDataType;
use gdal::DriverManager;

use gdal::Dataset;

use super::app::BasicApp;
use super::events::{dispatch_action, Action, DatasetCreationOptions, MessageHandler};
use crate::gis::commands::list_drivers;

pub fn create_dataset(options: &DatasetCreationOptions) -> Result<Dataset, ()> {
    eprintln!("{}", options.driver_name.trim().split('-').next().unwrap());
    let driver =
        DriverManager::get_driver_by_name(options.driver_name.split('-').next().unwrap().trim())
            .map_err(|err| {
                eprintln!("{:?}", err);
            })?;
    let dataset = driver
        .create(
            &options.file_name,
            options.raster_width as isize,
            options.raster_height as isize,
            options.raster.into(),
        )
        .map_err(|err| {
            eprintln!("{:?}", err);
        })?;
    Ok(dataset)
}

#[derive(PartialEq, Clone)]
pub enum State {
    Valid(ValidState),
    Invalid(String),
}
impl Default for State {
    fn default() -> Self {
        let drivers = list_drivers();
        State::Valid(ValidState {
            available_drivers: drivers,
        })
    }
}

#[derive(PartialEq, Clone)]
pub struct ValidState {
    pub available_drivers: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct NewDatasetSettings {
    pub file_name: String,
    pub driver: usize,
    pub options: Options,
    pub missed_requirements: Vec<String>,
}

impl NewDatasetSettings {
    pub fn to_options(&self, drivers: &[String]) -> DatasetCreationOptions {
        let (options, raster) = match self.options {
            Options::Vector(ref _a) => (RasterOptions::for_vector(), false),
            Options::Raster(ref options) => (options.clone(), true),
        };
        DatasetCreationOptions {
            file_name: self.file_name.clone(),
            driver_name: drivers[self.driver].clone(),
            raster_width: options.x,
            raster_height: options.y,
            raster_data_type: options.data_type,
            raster,
        }
    }
}

impl Default for NewDatasetSettings {
    fn default() -> Self {
        Self {
            file_name: "dataset".to_owned(),
            driver: 0,
            options: Options::Raster(RasterOptions::default()),
            missed_requirements: Vec::new(),
        }
    }
}
#[derive(PartialEq, Clone)]
pub struct RasterOptions {
    pub x: usize,
    pub y: usize,
    pub data_type: GdalDataType,
}

impl RasterOptions {
    fn for_vector() -> Self {
        Self {
            x: 0,
            y: 0,
            data_type: GdalDataType::Unknown,
        }
    }
}

impl Default for RasterOptions {
    fn default() -> Self {
        Self {
            x: 800,
            y: 800,
            data_type: GdalDataType::UInt8,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct VectorOptions;
impl Default for VectorOptions {
    fn default() -> Self {
        Self
    }
}

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
            State::Valid(state) => vec![(
                3,
                VNode::Custom(VComponent::new::<NewDatasetSettingsComponent, BasicApp>(
                    state.clone(),
                )),
            )],
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct NewDatasetSettingsComponent;
impl Component for NewDatasetSettingsComponent {
    type Props = ValidState;
    type State = NewDatasetSettings;
    type Message = Options;
    fn render(props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: "Drivers".to_owned(),
                }),
            ),
            (
                1,
                VNode::Select(VSelect {
                    options: props.available_drivers.clone(),
                    select: Some(|index, _, state| {
                        state.driver = index;
                        false
                    }),
                }),
            ),
            (
                2,
                VNode::Button(VButton {
                    text: match state.options {
                        Options::Vector(_) => "Make vector dataset",
                        Options::Raster(_) => "Make raster dataset",
                    }
                    .to_owned(),
                    click: Some(|_, state| {
                        state.options = match state.options {
                            Options::Vector(_) => Options::Raster(RasterOptions::default()),
                            Options::Raster(_) => Options::Vector(VectorOptions),
                        }
                    }),
                }),
            ),
            (
                3,
                VNode::Custom(match state.options {
                    Options::Vector(_) => VComponent::new::<NewVectorComponent, BasicApp>(()),
                    Options::Raster(_) => VComponent::new::<NewRasterComponent, BasicApp>(()),
                }),
            ),
            (
                4,
                VNode::Button(VButton {
                    text: "create".to_owned(),
                    click: Some(|props, state| {
                        eprintln!("clicked done");
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            Action::CreateDataset(state.to_options(&props.available_drivers)),
                        ));
                        dispatch_action(Action::CloseNewDatasetWindow);
                    }),
                }),
            ),
        ]
    }

    fn on_message(msg: &Self::Message, _props: &Self::Props, state: &mut Self::State) -> bool {
        state.options = msg.clone();
        true
    }
}

#[derive(Clone, PartialEq)]
pub struct NewRasterComponent;
impl Component for NewRasterComponent {
    type Props = ();
    type State = RasterOptions;
    fn render(_props: &Self::Props, _state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![]
    }
}

#[derive(Clone, PartialEq)]
pub struct NewVectorComponent;
impl Component for NewVectorComponent {
    type Props = ();
    type State = VectorOptions;
    fn render(_props: &Self::Props, _state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![]
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

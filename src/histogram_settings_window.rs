use cacao::{
    appkit::{
        window::{Window, WindowDelegate},
        App,
    },
    view::ViewController,
};
use cacao_framework::{Component, ComponentWrapper, Message, VButton, VLabel, VNode, VTextInput};

use crate::{
    app::BasicApp,
    events::{dispatch_action, Action, MessageHandler},
    graph::HistogramSettings,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OptionalHistogramSettings {
    /// The length the histogram should play for in milliseconds
    pub duration: Option<usize>,
    pub min_freq: Option<f64>,
    pub max_freq: Option<f64>,
}

impl OptionalHistogramSettings {
    fn merge_from(&self, settings: &HistogramSettings) -> HistogramSettings {
        HistogramSettings {
            duration: self.duration.unwrap_or(settings.duration),
            min_freq: self.min_freq.unwrap_or(settings.min_freq),
            max_freq: self.max_freq.unwrap_or(settings.max_freq),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct UpdateHistogramSettingsView;

impl Component for UpdateHistogramSettingsView {
    type Props = HistogramSettingsWrapper;
    type State = OptionalHistogramSettings;
    fn render(props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: "Duration (ms)".to_owned(),
                }),
            ),
            (
                1,
                VNode::TextInput(VTextInput {
                    initial_value: state
                        .duration
                        .unwrap_or(props.settings.duration)
                        .to_string(),
                    change: Some(|str, _, state| {
                        state.duration = str.parse().ok();
                        false
                    }),
                }),
            ),
            (
                2,
                VNode::Label(VLabel {
                    text: "Minimum frequency (Hz)".to_owned(),
                }),
            ),
            (
                3,
                VNode::TextInput(VTextInput {
                    initial_value: state
                        .min_freq
                        .unwrap_or(props.settings.min_freq)
                        .to_string(),
                    change: Some(|str, _, state| {
                        state.min_freq = str.parse().ok();
                        false
                    }),
                }),
            ),
            (
                4,
                VNode::Label(VLabel {
                    text: "Maximum frequency (Hz)".to_owned(),
                }),
            ),
            (
                5,
                VNode::TextInput(VTextInput {
                    initial_value: state
                        .max_freq
                        .unwrap_or(props.settings.max_freq)
                        .to_string(),
                    change: Some(|str, _, state| {
                        state.max_freq = str.parse().ok();
                        false
                    }),
                }),
            ),
            (
                6,
                VNode::Button(VButton {
                    text: "Done".to_owned(),
                    click: Some(|props, state| {
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            state.merge_from(&props.settings),
                        ));
                        dispatch_action(Action::CloseChangeHistogramSettings);
                    }),
                }),
            ),
        ]
    }
}

pub struct ChangeHistogramSettingsWindow {
    pub content: ViewController<ComponentWrapper<UpdateHistogramSettingsView, BasicApp>>,
}

impl ChangeHistogramSettingsWindow {
    pub fn new(position: usize, settings: HistogramSettings) -> Self {
        let content = ViewController::new(
            ComponentWrapper::<UpdateHistogramSettingsView, BasicApp>::new(
                HistogramSettingsWrapper::new(position, settings),
            ),
        );

        Self { content }
    }
}

impl MessageHandler<Message> for ChangeHistogramSettingsWindow {
    fn on_message(&self, message: &Message) {
        if let Some(delegate) = &self.content.view.delegate {
            delegate.on_message(message)
        };
    }
}

impl WindowDelegate for ChangeHistogramSettingsWindow {
    const NAME: &'static str = "ChangeHistogramSettingsWindow";

    fn did_load(&mut self, window: Window) {
        window.set_autosave_name("ChangeHistogramSettingsWindow");
        window.set_minimum_content_size(300, 100);
        window.set_title("Change settings for this histogram");
        window.set_content_view_controller(&self.content);
    }

    fn cancel(&self) {
        dispatch_action(Action::CloseChangeHistogramSettings);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramSettingsWrapper {
    settings: HistogramSettings,
    position: usize,
}

impl HistogramSettingsWrapper {
    fn new(position: usize, settings: HistogramSettings) -> Self {
        Self { position, settings }
    }
}

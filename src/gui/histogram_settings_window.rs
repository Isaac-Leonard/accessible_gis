use cacao::{
    appkit::{
        window::{Window, WindowDelegate},
        App,
    },
    view::ViewController,
};
use cacao_framework::{Component, ComponentWrapper, Message, VButton, VLabel, VNode, VTextInput};
use optional_struct::Applyable;

use super::{
    app::BasicApp,
    events::{dispatch_action, Action, MessageHandler},
};
use crate::{
    audio::graph::{HistogramSettings, OptionalHistogramSettings},
    raster::RasterIndex,
};

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
                        let mut settings = props.settings.clone();
                        state.clone().apply_to(&mut settings);
                        App::<BasicApp, Message>::dispatch_main(Message::custom(
                            HistogramSettingsWrapper {
                                settings,
                                index: props.index,
                            },
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
    pub fn new(settings: HistogramSettings, index: RasterIndex) -> Self {
        let content = ViewController::new(
            ComponentWrapper::<UpdateHistogramSettingsView, BasicApp>::new(
                HistogramSettingsWrapper::new(settings, index),
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
    pub settings: HistogramSettings,
    pub index: RasterIndex,
}

impl HistogramSettingsWrapper {
    fn new(settings: HistogramSettings, index: RasterIndex) -> Self {
        Self { settings, index }
    }
}

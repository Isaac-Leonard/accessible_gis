use std::time::Duration;

use super::{
    app::BasicApp,
    events::{dispatch_action, Action, MessageHandler},
};
use crate::{
    graph::{OptionalRasterGraphSettings, RasterGraphSettings},
    raster::RasterIndex,
};
use cacao::{
    appkit::{
        window::{Window, WindowDelegate},
        App,
    },
    view::ViewController,
};
use cacao_framework::{Component, ComponentWrapper, Message, VButton, VLabel, VNode, VTextInput};
use optional_struct::Applyable;

pub struct RasterGraphSettingsWindow {
    content: ViewController<ComponentWrapper<RasterGraphSettingsComponent, BasicApp>>,
}

impl RasterGraphSettingsWindow {
    pub fn new(settings: RasterGraphSettings, index: RasterIndex) -> Self {
        let content = ViewController::new(
            ComponentWrapper::<RasterGraphSettingsComponent, BasicApp>::new((settings, index)),
        );

        Self { content }
    }
}

impl MessageHandler<Message> for RasterGraphSettingsWindow {
    fn on_message(&self, message: &Message) {
        if let Some(ref delegate) = self.content.view.delegate {
            delegate.on_message(message);
        }
    }
}

impl WindowDelegate for RasterGraphSettingsWindow {
    const NAME: &'static str = "ChangeHistogramSettingsWindow";

    fn did_load(&mut self, window: Window) {
        window.set_autosave_name("ChangeHistogramSettingsWindow");
        window.set_minimum_content_size(300, 100);
        window.set_title("Change settings for this histogram");
        window.set_content_view_controller(&self.content);
    }

    fn cancel(&self) {
        dispatch_action(Action::CloseRasterSettings);
    }
}

#[derive(Clone, PartialEq)]
pub struct RasterGraphSettingsComponent;
impl Component for RasterGraphSettingsComponent {
    type Props = (RasterGraphSettings, RasterIndex);
    type State = OptionalRasterGraphSettings;
    fn render(
        (old, _): &Self::Props,
        new: &Self::State,
    ) -> Vec<(usize, cacao_framework::VNode<Self>)> {
        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: "Row Duration (ms)".to_owned(),
                }),
            ),
            (
                1,
                VNode::TextInput(VTextInput {
                    initial_value: new
                        .row_duration
                        .unwrap_or(old.row_duration)
                        .as_millis()
                        .to_string(),
                    change: Some(|str, _, state| {
                        state.row_duration = str.parse().ok().map(Duration::from_millis);
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
                    initial_value: new.min_freq.unwrap_or(old.min_freq).to_string(),
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
                    initial_value: new.max_freq.unwrap_or(old.max_freq).to_string(),
                    change: Some(|str, _, state| {
                        state.max_freq = str.parse().ok();
                        false
                    }),
                }),
            ),
            (
                6,
                VNode::Label(VLabel {
                    text: "Rows to play".to_owned(),
                }),
            ),
            (
                7,
                VNode::TextInput(VTextInput {
                    initial_value: new.rows.unwrap_or(old.rows).to_string(),
                    change: Some(|str, _, state| {
                        state.rows = str.parse().ok();
                        false
                    }),
                }),
            ),
            (
                8,
                VNode::Label(VLabel {
                    text: "Columns to play".to_owned(),
                }),
            ),
            (
                9,
                VNode::TextInput(VTextInput {
                    initial_value: new.rows.unwrap_or(old.cols).to_string(),
                    change: Some(|str, _, state| {
                        state.cols = str.parse().ok();
                        false
                    }),
                }),
            ),
            (
                1000,
                VNode::Button(VButton {
                    text: "Done".to_owned(),
                    click: Some(|(old_settings, index), state| {
                        let mut settings = old_settings.clone();
                        state.clone().apply_to(&mut settings);
                        App::<BasicApp, Message>::dispatch_main(Message::custom((
                            settings, *index,
                        )));
                        dispatch_action(Action::CloseRasterSettings);
                    }),
                }),
            ),
        ]
    }
}

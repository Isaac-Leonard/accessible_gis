use cacao::{
    appkit::window::{Window, WindowDelegate},
    button::Button,
    input::TextField,
    layout::{Layout, LayoutConstraint},
    text::Label,
    view::{View, ViewController, ViewDelegate},
};

use crate::{
    events::{dispatch_ui, Message},
    graph::HistogramSettings,
    layout::top_to_bottom,
};

pub struct UpdateHistogramSettingsView {
    duration_label: Label,
    duration_value: TextField,
    min_freq_label: Label,
    min_freq_value: TextField,
    max_freq_label: Label,
    max_freq_value: TextField,
    done_btn: Button,
    initial_data: HistogramSettingsWrapper,
}

impl UpdateHistogramSettingsView {
    fn new(settings: HistogramSettingsWrapper) -> Self {
        Self {
            duration_label: Label::new(),
            duration_value: TextField::default(),
            min_freq_label: Label::new(),
            min_freq_value: TextField::default(),
            max_freq_label: Label::new(),
            max_freq_value: TextField::default(),
            done_btn: Button::new("Done"),
            initial_data: settings,
        }
    }

    fn get_settings_value(&self) -> HistogramSettings {
        HistogramSettings {
            duration: self.duration_value.get_value().parse().unwrap(),
            min_freq: self.min_freq_value.get_value().parse().unwrap(),
            max_freq: self.max_freq_value.get_value().parse().unwrap(),
        }
    }

    fn on_message(&self, message: &Message) {
        match message {
            Message::ProcessHistogramSettings => {
                let position = self.initial_data.position;
                let settings = self.get_settings_value();
                dispatch_ui(Message::UpdateHistogramSettings(position, settings));
            }
            _ => {}
        }
    }
}

impl ViewDelegate for UpdateHistogramSettingsView {
    const NAME: &'static str = "UpdateHistogramSettingsView";
    fn did_load(&mut self, view: View) {
        view.add_subview(&self.duration_label);
        self.duration_label
            .set_text("Duration the graph should play for in milliseconds");
        view.add_subview(&self.duration_value);
        self.duration_value
            .set_text(&self.initial_data.settings.duration.to_string());

        view.add_subview(&self.min_freq_label);
        self.min_freq_label
            .set_text("The minimum frequency the of the graph in Hz");
        view.add_subview(&self.min_freq_value);
        self.min_freq_value
            .set_text(&self.initial_data.settings.min_freq.to_string());

        view.add_subview(&self.max_freq_label);
        self.max_freq_label
            .set_text("The maximum frequency the of the graph in Hz");
        view.add_subview(&self.max_freq_value);
        self.max_freq_value
            .set_text(&self.initial_data.settings.max_freq.to_string());

        view.add_subview(&self.done_btn);
        self.done_btn
            .set_action(|_| dispatch_ui(Message::ProcessHistogramSettings));
        LayoutConstraint::activate(&top_to_bottom(
            vec![
                &self.duration_label,
                &self.duration_value,
                &self.min_freq_label,
                &self.min_freq_value,
                &self.max_freq_label,
                &self.max_freq_value,
                &self.done_btn,
            ],
            &view,
            16.0,
        ))
    }
}

pub struct ChangeHistogramSettingsWindow {
    pub content: ViewController<UpdateHistogramSettingsView>,
}

impl ChangeHistogramSettingsWindow {
    pub fn new(position: usize, settings: HistogramSettings) -> Self {
        let content = ViewController::new(UpdateHistogramSettingsView::new(
            HistogramSettingsWrapper::new(position, settings),
        ));

        Self { content }
    }

    pub fn on_message(&self, message: &Message) {
        if let Some(delegate) = &self.content.view.delegate {
            delegate.on_message(message);
        }
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
        dispatch_ui(Message::CloseChangeHistogramSettings);
    }
}

#[derive(Debug, Clone)]
pub struct HistogramSettingsWrapper {
    settings: HistogramSettings,
    position: usize,
}

impl HistogramSettingsWrapper {
    fn new(position: usize, settings: HistogramSettings) -> Self {
        Self { position, settings }
    }
}

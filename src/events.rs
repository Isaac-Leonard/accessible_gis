use crate::app::BasicApp;

use crate::graph::HistogramSettings;

use cacao::appkit::App;
use cacao::view::View;

use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub enum Action {
    GotFile(PathBuf),
    InvalidFile(PathBuf),
    SetFeatureLabel(String),
    CloseChangeHistogramSettings,
    CloseSheet,
    OpenMainWindow,
    UpdateHistogramSettings(usize, HistogramSettings),
    SendAudioGraph(Vec<f64>, HistogramSettings),
    SendChangeHistogramSettings(usize, HistogramSettings),
}

#[derive(Clone, Debug)]
pub enum Click {
    SelectFile,
    DoneChangeHistogramSettings,
}

/// Dispatch a message on a background thread.
pub fn dispatch_action(message: Action) {
    App::<BasicApp, Message>::dispatch_main(Message::Action(message));
}

pub fn dispatch_click(message: Click) {
    App::<BasicApp, Message>::dispatch_main(Message::Click(message));
}

pub trait GetMessagible<Message> {
    /// Returns a list of references to all the sub components of this component with MessageHendler implemented for them
    /// Use dyn instead of a sized generic type as its almost certain there will need to be different types of subcomponents in the list
    fn get_messagable(&self) -> Vec<&dyn MessageHandler<Message>>;

    /// Called by the default MessageHandler impl for this trait
    fn on_message(&self, _message: &Message) {}
}

pub trait MessageHandler<T: Send + Sync> {
    fn on_message(&self, message: &T);
}

impl<M: Send + Sync, T: MessageHandler<M>> MessageHandler<M> for RwLock<T> {
    fn on_message(&self, message: &M) {
        if let Ok(handler) = self.read() {
            handler.on_message(message)
        };
    }
}

impl<M: Send + Sync, T: MessageHandler<M>> MessageHandler<M> for Option<T> {
    fn on_message(&self, message: &M) {
        if let Some(handler) = self {
            handler.on_message(message)
        };
    }
}

impl<M: Send + Sync, T: MessageHandler<M>> MessageHandler<M> for Box<T> {
    fn on_message(&self, message: &M) {
        self.as_ref().on_message(message)
    }
}

impl<M: Send + Sync, T: MessageHandler<M>> MessageHandler<M> for View<T> {
    fn on_message(&self, message: &M) {
        self.delegate.on_message(message)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Click(Click),
    Action(Action),
}

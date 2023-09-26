use crate::app::BasicApp;

use crate::graph::HistogramSettings;

use cacao::appkit::App;
use cacao::view::View;
use ndarray::Array2;

use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub enum Action {
    GotFile(PathBuf),
    InvalidFile(PathBuf),
    SendAudioGraph(Vec<f64>, HistogramSettings),
    SetFeatureLabel(String),
    CloseChangeHistogramSettings,
    SendChangeHistogramSettings(usize, HistogramSettings),
    CloseSheet,
    OpenMainWindow,
    UpdateHistogramSettings(usize, HistogramSettings),
    PlayRastaGraph(Array2<f64>, f64, f64, Option<f64>),
}

#[derive(Clone, Debug)]
pub enum Click {
    SelectFile,
    PlayHistogramGraph(usize),
    DoneChangeHistogramSettings,
    OpenChangeHistogramSettings(usize),
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

pub trait MessageHandler<T: Send + Sync + Clone> {
    fn on_message(&self, message: &T);
}

impl<M: Send + Sync + Clone, T: MessageHandler<M>> MessageHandler<M> for RwLock<T> {
    fn on_message(&self, message: &M) {
        if let Ok(handler) = self.read() {
            handler.on_message(message)
        };
    }
}

impl<M: Send + Sync + Clone, T: MessageHandler<M>> MessageHandler<M> for Option<T> {
    fn on_message(&self, message: &M) {
        if let Some(handler) = self {
            handler.on_message(message)
        };
    }
}

impl<M: Send + Sync + Clone, T: MessageHandler<M>> MessageHandler<M> for Box<T> {
    fn on_message(&self, message: &M) {
        self.as_ref().on_message(message)
    }
}

impl<M: Send + Sync + Clone, T: MessageHandler<M>> MessageHandler<M> for View<T> {
    fn on_message(&self, message: &M) {
        self.delegate.on_message(message)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Click(Click),
    Action(Action),
}

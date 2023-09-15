use crate::app::BasicApp;

use crate::graph::HistogramSettings;
use crate::raster::*;
use cacao::appkit::App;
use cacao::view::View;
use ndarray::Array2;

use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub enum Message {
    ClickedSelectFile,
    GotFile(PathBuf),
    InvalidFile(PathBuf),
    PlayHistogramGraph(usize),
    SendAudioGraph(Vec<f64>, HistogramSettings),
    RasterViewerAction(RasterViewerrMessage),
    SetFeatureLabel(String),
    CloseChangeHistogramSettings,
    OpenChangeHistogramSettings(usize),
    SendChangeHistogramSettings(usize, HistogramSettings),
    CloseSheet,
    OpenMainWindow,
    ProcessHistogramSettings,
    UpdateHistogramSettings(usize, HistogramSettings),
    PlayRastaGraph((usize, usize), Array2<u32>),
}

/// Dispatch a message on a background thread.
pub fn dispatch_ui(message: Message) {
    // println!("Dispatching UI message: {:?}", message);
    App::<BasicApp, Message>::dispatch_main(message);
}

pub trait GetMessagible<Message> {
    /// Returns a list of references to all the sub components of this component with MessageHendler implemented for them
    /// Use dyn instead of a sized generic type as its almost certain there will need to be different types of subcomponents in the list
    fn get_messagable(&self) -> Vec<&dyn MessageHandler<Message = Message>>;

    /// Called by the default MessageHandler impl for this trait
    fn on_message(&self, _message: &Message) {}
}

pub trait MessageHandler {
    type Message: Send + Sync;
    fn on_message(&self, message: &Self::Message);
}

impl<M: Send + Sync + Clone, T: MessageHandler<Message = M>> MessageHandler for RwLock<T> {
    type Message = M;
    fn on_message(&self, message: &Self::Message) {
        if let Ok(handler) = self.read() {
            handler.on_message(message)
        };
    }
}

impl<M: Send + Sync + Clone, T: MessageHandler<Message = M>> MessageHandler for Option<T> {
    type Message = M;
    fn on_message(&self, message: &Self::Message) {
        if let Some(handler) = self {
            handler.on_message(message)
        };
    }
}

impl<M: Send + Sync + Clone, T: MessageHandler<Message = M>> MessageHandler for Box<T> {
    type Message = M;
    fn on_message(&self, message: &Self::Message) {
        self.as_ref().on_message(message)
    }
}

impl<M: Send + Sync + Clone, T: MessageHandler<Message = M>> MessageHandler for View<T> {
    type Message = M;
    fn on_message(&self, message: &Self::Message) {
        self.delegate.on_message(message)
    }
}

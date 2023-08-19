use crate::app::BasicApp;

use crate::raster::*;
use cacao::appkit::App;

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Message {
    ClickedSelectFile,
    GotFile(PathBuf),
    InvalidFile(PathBuf),
    ToggleAudio,
    RasterViewerAction(RasterViewerrMessage),
}

/// Dispatch a message on a background thread.
pub fn dispatch_ui(message: Message) {
    println!("Dispatching UI message: {:?}", message);
    App::<BasicApp, Message>::dispatch_main(message);
}

pub trait GetMessagible<Message> {
    /// Returns a list of references to all the sub components of this component with MessageHendler implemented for them
    /// Use dyn instead of a sized generic type as its almost certain there will need to be different types of subcomponents in the list
    fn get_messagable(&self) -> Vec<&dyn MessageHandler<Message = Message>>;

    /// Called by the default MessageHandler impl for this trait
    fn on_message(&self, message: &Message) {}
}

pub trait MessageHandler {
    type Message: Send + Sync;
    fn on_message_mut(&mut self, message: Self::Message);
}

impl<T: GetMessagible<Message> + Sized> MessageHandler for T {
    type Message = Message;

    fn on_message_mut(&mut self, message: Self::Message) {
        for component in self.get_messagable_mut() {
            component.on_message_mut(message)
        }
    }
}

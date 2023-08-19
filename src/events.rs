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

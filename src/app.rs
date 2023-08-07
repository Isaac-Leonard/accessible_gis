use cacao::appkit::window::Window;
use cacao::appkit::{App, AppDelegate};
use cacao::notification_center::Dispatcher;
use cacao::view::ViewController;

use crate::views::{ContentView, Message};

pub struct BasicApp {
    window: Window,
    content_view: ViewController<ContentView>,
}

impl Default for BasicApp {
    fn default() -> Self {
        Self {
            window: Window::default(),
            content_view: ViewController::new(ContentView::default()),
        }
    }
}

impl Dispatcher for BasicApp {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        if let Some(delegate) = &self.content_view.view.delegate {
            delegate.on_ui_message(message);
        }
    }
}

impl AppDelegate for BasicApp {
    fn did_finish_launching(&self) {
        App::activate();
        self.window.set_minimum_content_size(400., 400.);
        self.window.set_title("Hello World!");
        self.window.set_movable_by_background(true);
        self.window.set_content_view_controller(&self.content_view);
        self.window.show();
        //        FileSelectPanel::new().show(|_| {});
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        true
    }
}
/// Dispatch a message on a background thread.
pub fn dispatch_ui(message: Message) {
    println!("Dispatching UI message: {:?}", message);
    App::<BasicApp, Message>::dispatch_main(message);
}

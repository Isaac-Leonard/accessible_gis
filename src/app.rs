use crate::events::Message;
use crate::windows::WindowManager;
use cacao::appkit::{App, AppDelegate};
use cacao::notification_center::Dispatcher;

#[derive(Default)]
pub struct BasicApp {
    pub window_manager: WindowManager,
}

impl Dispatcher for BasicApp {
    type Message = Message;

    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Self::Message) {
        self.window_manager.on_ui_message(message)
    }
}

impl AppDelegate for BasicApp {
    fn did_finish_launching(&self) {
        App::activate();
        self.window_manager.open_main()
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        true
    }
}

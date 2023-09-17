use crate::events::{Message, MessageHandler};
use crate::windows::WindowManager;
use cacao::appkit::{App, AppDelegate};
use cacao::notification_center::Dispatcher;

#[derive(Default)]
pub struct BasicApp {
    pub window_manager: WindowManager,
}

impl Dispatcher<Message> for BasicApp {
    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Message) {
        match &message {
            Message::Action(action) => self.window_manager.on_message(action),
            Message::Click(click) => self.window_manager.on_message(click),
        }
    }
}

impl Dispatcher<usize> for BasicApp {
    /// Handles button clicks from cacao_framework components that came over on the main (UI) thread.
    fn on_ui_message(&self, message: usize) {
        self.window_manager.on_message(&message)
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

use crate::events::{Action, MessageHandler};
use crate::windows::WindowManager;
use cacao::appkit::{App, AppDelegate};
use cacao::notification_center::Dispatcher;
use cacao_framework::Message as FrameworkMessage;

#[derive(Default)]
pub struct BasicApp {
    pub window_manager: WindowManager,
}

impl Dispatcher<Action> for BasicApp {
    /// Handles a message that came over on the main (UI) thread.
    fn on_ui_message(&self, message: Action) {
        self.window_manager.on_message(&message)
    }
}

impl Dispatcher<FrameworkMessage> for BasicApp {
    /// Handles button clicks from cacao_framework components that came over on the main (UI) thread.
    fn on_ui_message(&self, message: FrameworkMessage) {
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

use cacao::{
    appkit::window::{Window, WindowConfig, WindowDelegate},
    view::ViewController,
};
use std::sync::RwLock;

use crate::{
    events::{Action, Click, MessageHandler},
    graph::HistogramSettings,
    histogram_settings_window::ChangeHistogramSettingsWindow,
    views::MainView,
};

#[derive(Default)]
pub struct WindowManager {
    pub main: RwLock<Option<Window<MainWindow>>>,
    pub change_hist_settings: RwLock<Option<Window<ChangeHistogramSettingsWindow>>>,
}

/// A helper method to handle checking for window existence, and creating
/// it if not - then showing it.
fn open_or_show<T, F>(window: &RwLock<Option<Window<T>>>, vendor: F)
where
    T: WindowDelegate + 'static,
    F: Fn() -> (WindowConfig, T),
{
    let mut lock = window.write().unwrap();

    if let Some(win) = &*lock {
        win.show();
    } else {
        let (config, delegate) = vendor();
        let win = Window::with(config, delegate);
        win.show();
        *lock = Some(win);
    }
}

impl WindowManager {
    pub fn open_main(&self) {
        open_or_show(&self.main, || (WindowConfig::default(), MainWindow::new()));
    }

    /// When we run a sheet, we want to run it on our main window, which is all
    /// this helper is for.
    pub fn begin_sheet<W, F>(&self, window: &Window<W>, completion: F)
    where
        W: WindowDelegate + 'static,
        F: Fn() + Send + Sync + 'static,
    {
        let main = self.main.write().unwrap();

        if let Some(main_window) = &*main {
            main_window.begin_sheet(window, completion);
        }
    }

    /// Opens a "add file" window, which asks for a code and optional server to
    /// check against. This should, probably, be a sheet - but for now it's fine as a
    /// separate window until I can find time to port that API.
    pub fn open_histogram_settings(&self, position: usize, settings: HistogramSettings) {
        let callback = || {};

        let mut lock = self.change_hist_settings.write().unwrap();

        if let Some(win) = &*lock {
            self.begin_sheet(win, callback);
        } else {
            let window = Window::with(
                WindowConfig::default(),
                ChangeHistogramSettingsWindow::new(position, settings),
            );
            self.begin_sheet(&window, callback);
            *lock = Some(window);
        }
    }

    pub fn close_sheet(&self) {
        let mut add = self.change_hist_settings.write().unwrap();

        if let Some(add_window) = &*add {
            let main = self.main.write().unwrap();

            if let Some(main_window) = &*main {
                main_window.end_sheet(add_window);
            }
        }

        *add = None;
    }
}

impl MessageHandler<Action> for WindowManager {
    fn on_message(&self, message: &Action) {
        match message {
            Action::OpenMainWindow => {
                self.open_main();
            }

            Action::CloseSheet => {
                self.close_sheet();
            }

            Action::SendChangeHistogramSettings(position, settings) => {
                self.open_histogram_settings(*position, settings.clone());
            }

            Action::UpdateHistogramSettings(_, _) => {
                self.close_sheet();
            }

            message => {
                self.main.on_message(message);
                self.change_hist_settings.on_message(message);
            }
        }
    }
}

impl MessageHandler<Click> for WindowManager {
    fn on_message(&self, message: &Click) {
        dbg!(message);
        self.main.on_message(message);
        self.change_hist_settings.on_message(message);
    }
}

impl MessageHandler<usize> for WindowManager {
    fn on_message(&self, message: &usize) {
        self.main.on_message(message);
        self.change_hist_settings.on_message(message);
    }
}

pub struct MainWindow {
    pub window: Window,
    pub content: ViewController<MainView>,
}

impl MainWindow {
    fn new() -> Self {
        Self {
            window: Window::default(),
            content: ViewController::new(MainView::new()),
        }
    }
}

impl MessageHandler<Action> for MainWindow {
    fn on_message(&self, message: &Action) {
        self.content.view.on_message(message);
    }
}

impl MessageHandler<Click> for MainWindow {
    fn on_message(&self, message: &Click) {
        dbg!(message);
        self.content.view.on_message(message);
    }
}

impl MessageHandler<usize> for MainWindow {
    fn on_message(&self, message: &usize) {
        self.content.view.on_message(message);
    }
}

impl WindowDelegate for MainWindow {
    const NAME: &'static str = "MainWindow";

    fn did_load(&mut self, window: Window) {
        window.set_minimum_content_size(600, 400);
        window.set_title("Accessible GIS");
        window.set_content_view_controller(&self.content);
    }
}

/// A bunch of useful MessageHandler implementations to simplify code

impl<M: Send + Sync + Clone, T: MessageHandler<M>> MessageHandler<M> for Window<T> {
    fn on_message(&self, message: &M) {
        if let Some(delegate) = &self.delegate {
            delegate.on_message(message);
        }
    }
}

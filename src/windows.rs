use cacao::{
    appkit::window::{Window, WindowConfig, WindowDelegate},
    view::ViewController,
};
use cacao_framework::{ComponentWrapper, Message};
use std::sync::RwLock;

use crate::{
    app::BasicApp,
    events::{Action, MessageHandler},
    graph::{HistogramSettings, RasterGraphSettings},
    histogram_settings_window::ChangeHistogramSettingsWindow,
    new_dataset_window::NewDatasetWindow,
    raster::RasterIndex,
    raster_graph_settings_window::RasterGraphSettingsWindow,
    views::MainView,
};

#[derive(Default)]
pub struct WindowManager {
    pub main: RwLock<Option<Window<MainWindow>>>,
    pub change_hist_settings: RwLock<Option<Window<ChangeHistogramSettingsWindow>>>,
    pub raster_graph_settings: RwLock<Option<Window<RasterGraphSettingsWindow>>>,
    pub new_dataset: RwLock<Option<Window<NewDatasetWindow>>>,
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
    pub fn open_histogram_settings(&self, settings: HistogramSettings, index: RasterIndex) {
        let callback = || {};

        let mut lock = self.change_hist_settings.write().unwrap();

        if let Some(win) = &*lock {
            self.begin_sheet(win, callback);
        } else {
            let window = Window::with(
                WindowConfig::default(),
                ChangeHistogramSettingsWindow::new(settings, index),
            );
            self.begin_sheet(&window, callback);
            *lock = Some(window);
        }
    }

    pub fn open_raster_graph_settings(&self, settings: RasterGraphSettings, index: RasterIndex) {
        let callback = || {};

        let mut lock = self.raster_graph_settings.write().unwrap();

        if let Some(win) = &*lock {
            self.begin_sheet(win, callback);
        } else {
            let window = Window::with(
                WindowConfig::default(),
                RasterGraphSettingsWindow::new(settings, index),
            );
            self.begin_sheet(&window, callback);
            *lock = Some(window);
        }
    }

    pub fn open_new_dataset(&self) {
        let callback = || {};

        let mut lock = self.new_dataset.write().unwrap();

        if let Some(win) = &*lock {
            self.begin_sheet(win, callback);
        } else {
            let window = Window::with(WindowConfig::default(), NewDatasetWindow::new());
            self.begin_sheet(&window, callback);
            *lock = Some(window);
        }
    }

    pub fn close_histogram_settings(&self) {
        let mut add = self.change_hist_settings.write().unwrap();

        if let Some(add_window) = &*add {
            let main = self.main.write().unwrap();

            if let Some(main_window) = &*main {
                main_window.end_sheet(add_window);
            }
        }

        *add = None;
    }

    pub fn close_raster_graph_settings(&self) {
        eprint!("here");
        let mut add = self.raster_graph_settings.write().unwrap();

        if let Some(add_window) = &*add {
            let main = self.main.write().unwrap();

            if let Some(main_window) = &*main {
                main_window.end_sheet(add_window);
            }
        }

        *add = None;
    }

    pub fn close_new_dataset(&self) {
        let mut new_dataset_window = self.new_dataset.write().unwrap();

        if let Some(add_window) = &*new_dataset_window {
            let main = self.main.write().unwrap();

            if let Some(main_window) = &*main {
                main_window.end_sheet(add_window);
            }
        }

        *new_dataset_window = None;
    }
}

impl MessageHandler<Action> for WindowManager {
    fn on_message(&self, message: &Action) {
        match message {
            Action::CloseChangeHistogramSettings => self.close_histogram_settings(),
            Action::CloseRasterSettings => self.close_raster_graph_settings(),
            Action::CloseNewDatasetWindow => self.close_new_dataset(),
            Action::OpenMainWindow => {
                self.open_main();
            }

            Action::SendChangeHistogramSettings(settings, index) => {
                self.open_histogram_settings(settings.clone(), *index);
            }
            Action::SendChangeRasterGraphSettings(settings, index) => {
                self.open_raster_graph_settings(settings.clone(), *index);
            }
            Action::OpenNewDatasetWindow => self.open_new_dataset(),
            _ => {}
        }
    }
}

impl MessageHandler<Message> for WindowManager {
    fn on_message(&self, message: &Message) {
        dbg!(message);
        self.main.on_message(message);
        self.change_hist_settings.on_message(message);
        self.raster_graph_settings.on_message(message);
        self.new_dataset.on_message(message);
    }
}

pub struct MainWindow {
    pub window: Window,
    pub content: ViewController<ComponentWrapper<MainView, BasicApp>>,
}

impl MainWindow {
    fn new() -> Self {
        Self {
            window: Window::default(),
            content: ViewController::new(ComponentWrapper::new(())),
        }
    }
}

impl MessageHandler<Message> for MainWindow {
    fn on_message(&self, message: &Message) {
        if let Some(ref delegate) = self.content.view.delegate {
            delegate.on_message(message);
        }
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

impl<M: Send + Sync, T: MessageHandler<M>> MessageHandler<M> for Window<T> {
    fn on_message(&self, message: &M) {
        if let Some(delegate) = &self.delegate {
            delegate.on_message(message);
        }
    }
}

#![feature(sort_floats)]
#![feature(result_option_inspect)]
mod graph;
mod app;
mod audio;
mod events;
mod list_view;
mod raster;
mod vector;
mod views;

use cacao::{appkit::App, view::ViewController};
use views::MainView;

fn main() {
    App::new(
        "com.hello.world",
        app::BasicApp {
            window: cacao::appkit::window::Window::default(),
            content: ViewController::new(MainView::new()),
        },
    )
    .run();
}

#![feature(result_option_inspect)]
mod app;
mod audio;
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

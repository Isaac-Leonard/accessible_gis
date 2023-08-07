mod app;
mod audio;
mod views;
use cacao::{appkit::App, view::ViewController};
use views::ContentView;

fn main() {
    App::new(
        "com.hello.world",
        app::BasicApp {
            window: cacao::appkit::window::Window::default(),
            content_view: ViewController::new(ContentView::default()),
        },
    )
    .run();
}

mod app;
mod audio;
mod views;
use cacao::appkit::App;

fn main() {
    App::new("com.hello.world", app::BasicApp::default()).run();
}

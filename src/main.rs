use cacao::appkit::window::Window;
use cacao::appkit::{App, AppDelegate};
use cacao::filesystem::FileSelectPanel;
use cacao::{button, view};
use shapefile::read;
use std::env;
use std::fs;

#[derive(Default)]
struct BasicApp {
    window: Window,
}

impl AppDelegate for BasicApp {
    fn did_finish_launching(&self) {
        App::activate();
        self.window.set_minimum_content_size(400., 400.);
        self.window.set_title("Hello World!");
        self.window.set_movable_by_background(true);
        let view_content = view::View::new();
        self.window.set_content_view(&view_content);
        self.window.show();
        //        FileSelectPanel::new().show(|_| {});
    }
}

fn main() {
    App::new("com.hello.world", BasicApp::default()).run();
}

fn main_2() {
    let args = env::args().collect::<Vec<_>>();
    let file = args.get(1).expect("No file provided");
    let shape_file = read(file).expect("Could not get shapefile data");
    for (shape, record) in shape_file {
        println!("{}", shape.shapetype());
        for (key, value) in record {
            println!("  {}: {}", key, value)
        }
    }
}

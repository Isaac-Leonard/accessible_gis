use crate::audio::{get_audio, AudioMessage};
use crate::events::{dispatch_ui, Message, MessageHandler};

use crate::raster::*;
use crate::vector::VectorLayerView;
use cacao::filesystem::FileSelectPanel;
use cacao::foundation::NSURL;
use cacao::layout::{Layout, LayoutConstraint};

use cacao::notification_center::Dispatcher;
use cacao::text::Label;
use cacao::view::{View, ViewDelegate};
use cacao::{button::Button, view};
use gdal::vector::LayerAccess;
use gdal::Dataset;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

pub struct MainView {
    content: view::View,
    button: Button,
    dataset_view: Rc<RefCell<Option<View<DatasetView>>>>,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            content: View::new(),
            button: Button::new("Select file"),
            dataset_view: Rc::new(RefCell::new(None)),
        }
    }

    pub fn on_message(&self, message: &Message) {
        match message {
            Message::SetFeatureLabel(_)
            | Message::UpdateHistogramSettings(_, _)
            | Message::ProcessHistogramSettings
            | Message::PlayHistogramGraph(_)
            | Message::CloseChangeHistogramSettings
            | Message::OpenChangeHistogramSettings(_)
            | Message::SendChangeHistogramSettings(_, _)
            | Message::CloseSheet
            | Message::OpenMainWindow
            | Message::SendAudioGraph(_, _)
            | Message::PlayRastaGraph(_, _)
            | Message::RasterViewerAction(_) => {
                let dataset_view = self.dataset_view.borrow_mut();
                dataset_view
                    .as_ref()
                    .and_then(|x| x.delegate.as_ref())
                    .inspect(|x| x.on_message(message));
            }
            Message::ClickedSelectFile => FileSelectPanel::new().show(file_selection_handler),
            Message::GotFile(path) => {
                let dataset = Dataset::open(path).expect("Could'nt read file");
                let mut dataset_view = self.dataset_view.borrow_mut();
                let sub_view = View::with(DatasetView::new(dataset));
                self.content.add_subview(&sub_view);
                *dataset_view = Some(sub_view);
            }
            Message::InvalidFile(_) => {}
        }
    }
}

impl Default for MainView {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DatasetView {
    content: view::View,
    sub_views: Rc<RefCell<Vec<View<LayerView>>>>,
    audio: Sender<AudioMessage>,
    dataset: Rc<RefCell<Dataset>>,
    spatial_reference_label: Label,
}

impl DatasetView {
    fn new(dataset: Dataset) -> Self {
        let audio = get_audio();
        let view = Self {
            content: View::new(),
            dataset: Rc::new(RefCell::new(dataset)),
            audio,
            sub_views: Rc::new(RefCell::new(Vec::new())),
            spatial_reference_label: Label::new(),
        };
        {
            let dataset = view.dataset.borrow_mut();
            let mut sub_views = Vec::new();
            let layers = dataset.layers();
            let mut last_position = 0;
            for layer in layers {
                let vector_view = View::with(VectorLayerView::new(layer, None));
                vector_view.set_background_color(cacao::color::Color::SystemRed);
                sub_views.push(View::with(LayerView::Vector(vector_view)));
                last_position += 1;
            }
            // TODO: Lets try replace with a proper iterator
            for i in 1..=dbg!(dataset.raster_count()) {
                let band = dataset.rasterband(i).unwrap();
                let raster_view =
                    View::with(RasterLayerView::new(&band, i as usize + last_position));
                raster_view.set_background_color(cacao::color::Color::SystemRed);
                sub_views.push(View::with(LayerView::Raster(raster_view)));
            }
            view.sub_views.borrow_mut().append(&mut sub_views);
        }
        view
    }

    fn update_new_label(&self, labeled_by: Option<String>) {
        let mut sub_views = Vec::new();
        let dataset = self.dataset.borrow_mut();
        let layers = dataset.layers();
        let mut last_position = 0;
        for layer in layers {
            let vector_view = View::with(VectorLayerView::new(layer, labeled_by.clone()));
            vector_view.set_background_color(cacao::color::Color::SystemRed);
            sub_views.push(View::with(LayerView::Vector(vector_view)));
            last_position += 1;
        }
        // TODO: Lets try replace with a proper iterator
        for i in 1..=dataset.raster_count() {
            let band = dataset.rasterband(i).unwrap();
            let raster_view = View::with(RasterLayerView::new(&band, i as usize + last_position));
            raster_view.set_background_color(cacao::color::Color::SystemRed);
            let view = View::with(LayerView::Raster(raster_view));
            sub_views.push(view);
        }
        let mut old_sub_views = self.sub_views.borrow_mut();
        old_sub_views.clear();
        old_sub_views.append(&mut sub_views);
        for view in old_sub_views.iter() {
            self.content.add_subview(view);
        }
    }
}

pub enum LayerView {
    Vector(View<VectorLayerView>),
    Raster(View<RasterLayerView>),
}

impl ViewDelegate for LayerView {
    const NAME: &'static str = "LayerView";
    fn did_load(&mut self, view: View) {
        match self {
            Self::Raster(raster) => view.add_subview(raster),
            Self::Vector(vector) => view.add_subview(vector),
        }
        LayoutConstraint::activate(&[
            view.height.constraint_equal_to_constant(300.),
            view.width.constraint_equal_to_constant(600.),
        ])
    }
}

impl LayerView {
    fn on_message(&self, message: &Message) {
        match self {
            Self::Vector(_vector_view) => {}
            Self::Raster(raster_view) => {
                if let Some(ref view) = raster_view.delegate {
                    view.as_ref().on_message(message)
                }
            }
        }
    }
}

impl ViewDelegate for MainView {
    const NAME: &'static str = "SafeAreaView";

    fn did_load(&mut self, view: View) {
        self.button
            .set_action(|_| dispatch_ui(Message::ClickedSelectFile));
        self.button.set_key_equivalent("c");
        self.content.add_subview(&self.button);
        view.add_subview(&self.content);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        cacao::layout::LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top),
            self.content
                .leading
                .constraint_equal_to(&view.safe_layout_guide.leading),
            self.content
                .trailing
                .constraint_equal_to(&view.safe_layout_guide.trailing),
            self.content
                .bottom
                .constraint_equal_to(&view.safe_layout_guide.bottom),
        ])
    }
}

impl ViewDelegate for DatasetView {
    const NAME: &'static str = "DataSetView";

    fn did_load(&mut self, view: View) {
        for sub_view in self.sub_views.borrow().iter() {
            self.content.add_subview(sub_view)
        }
        self.spatial_reference_label.set_text(
            self.dataset
                .borrow()
                .spatial_ref()
                .unwrap_or_else(|_| {
                    self.dataset
                        .borrow()
                        .layer(0)
                        .unwrap()
                        .spatial_ref()
                        .unwrap()
                })
                .to_pretty_wkt()
                .unwrap(),
        );
        self.content.add_subview(&self.spatial_reference_label);
        view.add_subview(&self.content);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top),
            self.content
                .leading
                .constraint_equal_to(&view.safe_layout_guide.leading),
            self.content
                .trailing
                .constraint_equal_to(&view.safe_layout_guide.trailing),
            self.content
                .bottom
                .constraint_equal_to(&view.safe_layout_guide.bottom),
            self.spatial_reference_label
                .top
                .constraint_equal_to(&self.content.top),
            self.spatial_reference_label
                .leading
                .constraint_equal_to(&self.content.leading),
            self.spatial_reference_label
                .height
                .constraint_equal_to_constant(40.0),
            self.spatial_reference_label
                .width
                .constraint_equal_to_constant(80.0),
        ])
    }
}

fn file_selection_handler(paths: Vec<NSURL>) {
    if paths.is_empty() {
        // User canceled
        return;
    }
    let path = paths[0].pathbuf();
    // Simplistic check for vector file
    if path.is_dir() {
        dispatch_ui(Message::InvalidFile(path));
        return;
    }
    dispatch_ui(Message::GotFile(path))
}

impl DatasetView {
    fn on_message(&self, message: &Message) {
        match message {
            Message::SendAudioGraph(graph, settings) => self
                .audio
                .send(AudioMessage::PlayHistogram(graph.clone(), settings.clone()))
                .unwrap(),
            Message::PlayRastaGraph(size, data) => self
                .audio
                .send(AudioMessage::PlayRasta(size.clone(), data.clone()))
                .unwrap(),
            Message::SetFeatureLabel(name) => self.update_new_label(Some(name.clone())),
            Message::RasterViewerAction(_)
            | Message::SendAudioGraph(_, _)
            | Message::PlayHistogramGraph(_)
            | Message::OpenChangeHistogramSettings(_)
            | Message::UpdateHistogramSettings(_, _) => {
                let views = self.sub_views.borrow();
                views
                    .iter()
                    .filter_map(|v| v.delegate.as_ref())
                    .for_each(|view| view.on_message(message));
            }
            Message::InvalidFile(_) => {}
            Message::GotFile(_) => {}
            Message::ClickedSelectFile => {}
            Message::CloseChangeHistogramSettings => {}
            _ => {}
        }
    }
}

pub struct DatasetContainer {
    dataset: Dataset,
}

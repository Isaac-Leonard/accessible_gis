use cacao::{
    appkit::window::{Window, WindowDelegate},
    view::ViewController,
};
use cacao_framework::{Component, ComponentWrapper};

use crate::{
    app::BasicApp,
    events::{dispatch_action, Action},
    graph::{OptionalRasterGraphSettings, RasterGraphSettings},
};

pub struct RasterGraphSettingsWindow {
    content: ViewController<ComponentWrapper<RasterGraphSettingsComponent, BasicApp>>,
}

impl RasterGraphSettingsWindow {
    pub fn new(position: usize, settings: RasterGraphSettings) -> Self {
        let content = ViewController::new(
            ComponentWrapper::<RasterGraphSettingsComponent, BasicApp>::new(settings),
        );

        Self { content }
    }
}

impl WindowDelegate for RasterGraphSettingsWindow {
    const NAME: &'static str = "ChangeHistogramSettingsWindow";

    fn did_load(&mut self, window: Window) {
        window.set_autosave_name("ChangeHistogramSettingsWindow");
        window.set_minimum_content_size(300, 100);
        window.set_title("Change settings for this histogram");
        window.set_content_view_controller(&self.content);
    }

    fn cancel(&self) {
        dispatch_action(Action::CloseRasterSettings);
    }
}

#[derive(Clone, PartialEq)]
pub struct RasterGraphSettingsComponent;
impl Component for RasterGraphSettingsComponent {
    type Props = RasterGraphSettings;
    type State = OptionalRasterGraphSettings;
    fn render(
        props: &Self::Props,
        state: &Self::State,
    ) -> Vec<(usize, cacao_framework::VNode<Self>)> {
        Vec::new()
    }
}

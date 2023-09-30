use cacao::view::ViewController;
use cacao_framework::{Component, ComponentWrapper};

use crate::{
    app::BasicApp,
    graph::{OptionalRasterGraphSettings, RasterGraphSettings},
};

pub struct RasterGraphSettingsWindow {
    content: ViewController<ComponentWrapper<RasterGraphSettingsComponent, BasicApp>>,
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

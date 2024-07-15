use enum_dispatch::enum_dispatch;
pub use geo_types::Geometry as GeoGeometry;
use geo_types::Point;
use serde::{Deserialize, Serialize};
use specta::*;
use strum::EnumDiscriminants;

use crate::gdal_if::LayerIndex;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ToolList {
    tools: Vec<ToolData>,
    selected_tool: Option<usize>,
}

impl ToolList {
    pub fn pop(&mut self) -> Option<ToolData> {
        self.tools.pop()
    }
    pub fn add_tool(&mut self, tool: ToolDataDiscriminants) {
        self.tools.push(tool.into());
        self.selected_tool = Some(self.tools.len() - 1);
    }

    pub fn get_first_tool_for_layer(
        &mut self,
        dataset: usize,
        layer: LayerIndex,
    ) -> Option<&mut ToolData> {
        self.tools
            .iter_mut()
            .find(move |tool| tool.is_tool_for(dataset, layer.clone()))
    }
}

#[enum_dispatch(Tool)]
#[derive(EnumDiscriminants, Clone, Debug, PartialEq)]
#[strum_discriminants(derive(Serialize, Deserialize, Type))]
pub enum ToolData {
    TraceGeometries(TraceGeometries),
}

impl From<ToolDataDiscriminants> for ToolData {
    fn from(value: ToolDataDiscriminants) -> Self {
        match value {
            ToolDataDiscriminants::TraceGeometries => ToolData::TraceGeometries(Default::default()),
        }
    }
}

impl ToolData {
    pub fn as_trace_geometry(&mut self) -> Option<&mut TraceGeometries> {
        match self {
            Self::TraceGeometries(ref mut tool) => Some(tool),
            _ => None,
        }
    }
}

#[enum_dispatch]
pub trait Tool {
    fn is_tool_for(&self, dataset: usize, layer: LayerIndex) -> bool;
    fn pass_point(&mut self, point: Point) -> Result<(), String> {
        Err("Not implemented".to_string())
    }

    fn save_vector(&mut self, _name: String, driver: String) -> Result<(), String> {
        Err("Unimplemented".to_string())
    }

    fn add_layer(&mut self, _dataset: usize, layer: usize) -> Result<(), String> {
        Err("This tool doesn't take layers".to_owned())
    }
    fn add_band(&mut self, _dataset: usize, band: usize) -> Result<(), String> {
        Err("This tool doesn't take bands".to_owned())
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TraceGeometries {
    pub dataset: Option<usize>,
    pub band: Option<usize>,
    pub points: Vec<Point>,
    pub geometries: Vec<GeoGeometry>,
}

impl Tool for TraceGeometries {
    fn is_tool_for(&self, dataset: usize, layer: LayerIndex) -> bool {
        self.dataset == Some(dataset) && self.band == layer.as_raster().copied()
    }
}

impl TraceGeometries {}

use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::ErrorDetails,
    gdal_if::{LayerIndexDiscriminants, WrappedDataset},
};

use super::{
    configurable_tools::{Input, ParameterValue, ToolOutput},
    dataset_collection::NonEmptyDelegatorImpl,
    gis::combined::DatasetLayerIndex,
    projects::Project,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "value")]
pub enum WorkflowInputIndex {
    Raw(usize),
    ToolResult(ToolResult),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolResult {
    tool: usize,
    output: usize,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct NewWorkflow {
    label: String,
    pub inputs: Vec<NewWorkflowInputDescriptor>,
    pub tools: Vec<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct Workflow {
    label: String,
    inputs: Vec<WorkflowInputDescriptor>,
    pub tools: Vec<Uuid>,
}

impl Workflow {
    // TODO: add validation
    pub fn create(workflow: NewWorkflow) -> Self {
        let mut inputs = Vec::new();
        for input in workflow.inputs {}
        Self {
            label: workflow.label,
            inputs,
            tools: workflow.tools,
        }
    }
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct RuntimeInputs {
    inputs: Vec<WorkflowInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct WorkflowConnection {
    tool: Uuid,
    parameter: Uuid,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct NewWorkflowInputDescriptor {
    label: String,
    value: WorkflowInputValueDescriptor,
    connections: Vec<WorkflowConnection>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct WorkflowInputDescriptor {
    id: Uuid,
    label: String,
    value: WorkflowInputValueDescriptor,
    connections: Vec<WorkflowConnection>,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, specta::Type, strum::EnumTryAs, strum::EnumDiscriminants,
)]
#[serde(tag = "type", content = "value")]
#[strum_discriminants(derive(Serialize, Deserialize, specta::Type, strum::EnumIter))]
pub enum WorkflowInputValueDescriptor {
    Float,
    Int,
    String,
    Layer(LayerIndexDiscriminants),
    Option(Vec<String>),
    Flag,
    File,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct WorkflowInput {
    id: Uuid,
    value: WorkflowInputValue,
}

#[derive(Clone, Debug, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum WorkflowInputValue {
    Float(f64),
    Int(i64),
    String(String),
    Option(String),
    Flag(bool),
    File(FileValue),
}

#[derive(Clone, Debug, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum FileValue {
    Temp,
    Custom(PathBuf),
}

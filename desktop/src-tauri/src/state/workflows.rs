use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::ErrorDetails, gdal_if::WrappedDataset};

use super::{
    configurable_tools::{Input, ParameterValue, ToolOutput},
    dataset_collection::NonEmptyDelegatorImpl,
    gis::combined::DatasetLayerIndex,
    projects::Project,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolWrapper {
    tool: usize,
    inputs: Vec<WorkflowInputIndex>,
}

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Workflow {
    inputs: Vec<Input>,
    pub tool_calls: Vec<ToolWrapper>,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct RuntimeInputs {
    inputs: Vec<WorkflowValue>,
}

pub enum WorkflowInputDescriptor {
    Input(Input),
    Tool(PathBuf),
}

fn run_workflow(
    project: &mut Project,
    workflow: Workflow,
    inputs: RuntimeInputs,
) -> Result<(), ErrorDetails> {
    let mut tool_outputs: Vec<ToolOutput> = Vec::new();
    let mut dataset_storage = Vec::new();
    let mut datasets = project
        .datasets
        .iter_mut()
        .map(|ds| &mut ds.dataset)
        .collect_vec();
    for call in workflow.tool_calls {
        let tool = project.tools.get(call.tool).unwrap();
        let mut inputs = Vec::new();
        for index in call.inputs.iter() {
            let input = match index {
                WorkflowInputIndex::Raw(index) => {
                    WorkflowInputDescriptor::Input(workflow.inputs.get(*index).unwrap().clone())
                }
                WorkflowInputIndex::ToolResult(ToolResult { tool, output }) => {
                    let tool_output = tool_outputs.get(*tool).ok_or_else(|| {
                        ErrorDetails::Other(
                            "Tried to run tool before tool it requires for input ran".to_string(),
                        )
                    })?;
                    let path = tool_output.files.get(*output).ok_or_else(|| {
                        ErrorDetails::Other("Tried to get non existant output of tool".to_string())
                    })?;
                    let dataset =
                        WrappedDataset::open(path).map_err(ErrorDetails::OpenDatasetError)?;
                    dataset_storage.push(dataset);
                    WorkflowInputDescriptor::Tool(path.clone())
                }
            };
            inputs.push(input)
        }
        let expected_inputs = tool.get_expected_input_parameters();
        let output = todo![];
        tool_outputs.push(output)
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum WorkflowValue {
    Float(f64),
    Int(i64),
    String(String),
    Layer(DatasetLayerIndex),
    Dataset(usize),
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

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct WorkflowInput {
    id: Uuid,
    value: WorkflowValue,
}

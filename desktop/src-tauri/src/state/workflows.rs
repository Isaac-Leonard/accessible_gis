// my_module.rs
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::ErrorDetails, gdal_if::LayerIndexDiscriminants};

use super::{
    projects::Project,
    tools::{
        NewToolInput, SavedToolOutputAction, Tool, ToolInputDescriptor, ToolInputType,
        ToolParameter, ToolParameterValue, ToolPresetParameterValue,
    },
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolResult {
    tool: usize,
    output: usize,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct NewWorkflow {
    label: String,
    pub inputs: Vec<NewWorkflowInputDescriptor>,
    pub tools: Vec<NewToolCall>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct Workflow {
    pub label: String,
    pub inputs: Vec<WorkflowInputDescriptor>,
    pub tools: Vec<ToolCall>,
}

impl Workflow {
    // TODO: add validation
    pub fn create(workflow: NewWorkflow) -> Self {
        let inputs = workflow
            .inputs
            .into_iter()
            .map(WorkflowInputDescriptor::new)
            .collect_vec();
        let tools = workflow
            .tools
            .into_iter()
            .map(|tool| ToolCall {
                tool: tool.tool,
                inputs: tool
                    .inputs
                    .into_iter()
                    .map(|input| WorkflowConnection {
                        parameter: input.parameter,
                        input: inputs[input.input].id,
                    })
                    .collect_vec(),
            })
            .collect();
        Self {
            label: workflow.label,
            inputs,
            tools,
        }
    }

    fn run_workflow(
        &self,
        inputs: Vec<WorkflowInput>,
        project: &mut Project,
    ) -> Result<(), ErrorDetails> {
        for tool_call in &self.tools {
            let tool = project
                .tools
                .iter()
                .find(|tool| tool.get_id() == tool_call.tool)
                .ok_or_else(|| {
                    ErrorDetails::Other("Could not get tool for workflow".to_string())
                })?;

            let workflow_inputs = tool_call
                .inputs
                .iter()
                .map(|connection| {
                    let expected = self
                        .inputs
                        .iter()
                        .find(|expected| expected.id == connection.input)
                        .unwrap()
                        .clone();
                    let got = inputs
                        .iter()
                        .find(|input| input.id == connection.input)
                        .map(|input| input.value.clone());

                    ToolParameter {
                        id: connection.parameter,
                        value: got
                            .unwrap_or_else(|| expected.value.try_as_preset().unwrap().into()),
                    }
                })
                .collect_vec();

            project.tool_outputs.push(SavedToolOutputAction {
                read: true,
                tool: tool.get_label(),
                output: tool.run(workflow_inputs, &mut project.datasets)?,
                id: Uuid::new_v4(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct WorkflowConnection {
    pub parameter: Uuid,
    pub input: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct NewWorkflowConnection {
    pub parameter: Uuid,
    pub input: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ToolCall {
    pub tool: Uuid,
    pub inputs: Vec<WorkflowConnection>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct NewToolCall {
    pub tool: Uuid,
    pub inputs: Vec<NewWorkflowConnection>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum WorkflowInputDescriptorValue {
    Preset(ToolPresetParameterValue),
    Runtime(WorkflowInputRuntimeValueDescriptor),
}

#[derive(Clone, Debug, Deserialize, Serialize, specta::Type)]
pub struct WorkflowInputRuntimeValueDescriptor {
    param_type: ToolInputType,
    optional: bool,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct NewWorkflowInputDescriptor {
    pub label: String,
    pub value: WorkflowInputDescriptorValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct WorkflowInputDescriptor {
    pub id: Uuid,
    pub label: String,
    pub value: WorkflowInputDescriptorValue,
}

impl WorkflowInputDescriptor {
    pub fn new(input: NewWorkflowInputDescriptor) -> Self {
        Self {
            label: input.label,
            value: input.value,
            id: Uuid::new_v4(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct WorkflowInput {
    pub id: Uuid,
    pub value: ToolParameterValue,
}

#[derive(Clone, Debug, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum FileValue {
    Temp,
    Custom(PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct UiWorkflow {
    id: Uuid,
    label: String,
    inputs: Vec<UiWorkflowInputDescriptor>,
}

/// Identical to `ToolRuntimeInputDescriptor ` for now
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct UiWorkflowInputDescriptor {
    id: Uuid,
    label: String,
    param_type: ToolInputType,
    optional: bool,
}

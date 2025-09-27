// my_module.rs
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_specta::Event;
use uuid::Uuid;

use crate::{commands::MessageEvent, errors::ErrorDetails, gdal_if::LayerIndexDiscriminants};

use super::{
    AppDataSync, AppState,
    projects::Project,
    tools::{
        NewToolInput, ReturnedToolOutput, SavedToolOutputAction, Tool, ToolInputDescriptor,
        ToolInputType, ToolOutput, ToolParameter, ToolParameterValue, ToolPresetParameterValue,
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
    pub id: Uuid,
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
            id: Uuid::new_v4(),
            label: workflow.label,
            inputs,
            tools,
        }
    }

    pub fn run_workflow(
        &self,
        inputs: Vec<WorkflowInput>,
        project: &mut Project,
        app: &AppHandle,
    ) -> Result<(), ErrorDetails> {
        let tool_calls: Vec<_> = self
            .tools
            .iter()
            .map(|tool_call| {
                let tool = project
                    .tools
                    .iter()
                    .find(|tool| tool.get_id() == tool_call.tool)
                    .ok_or_else(|| {
                        ErrorDetails::Other("Could not get tool for workflow".to_string())
                    })?
                    .clone();

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
                let params =
                    tool.parse_input_parameters(workflow_inputs, &mut project.datasets, app)?;
                Ok::<_, ErrorDetails>((tool, params))
            })
            .try_collect()?;
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let state = app.state::<AppDataSync>();
            for (tool, params) in tool_calls {
                let result = tool.execute(params);
                match result {
                    Ok(result) => {
                        state.with_project(|project| {
                            project.tool_outputs.push(SavedToolOutputAction {
                                read: false,
                                tool: tool.get_label(),
                                output: ToolOutput {
                                    returned_output: result,
                                    files: Vec::new(),
                                },
                                id: Uuid::new_v4(),
                            })
                        });
                    }
                    Err(err) => {
                        state.with_lock(|state| {
                            state.errors.push(err.into());
                            state.errors.push(
                                ErrorDetails::Other(format!(
                                    "Failed to finish workflow due to error in the {} tool",
                                    tool.get_label()
                                ))
                                .into(),
                            );
                        });
                        return;
                    }
                };
                MessageEvent.emit(&app);
            }
            // This should not be an error but there's currently no better notification mechinism
            state.with_lock(|state| {
                state
                    .errors
                    .push(ErrorDetails::Other("Project done".to_string()).into())
            });
            MessageEvent.emit(&app);
        });
        Ok(())
    }

    pub fn for_ui(&self) -> UiWorkflow {
        UiWorkflow {
            id: self.id,
            label: self.label.clone(),
            inputs: self
                .inputs
                .iter()
                .filter_map(|input| {
                    let value = input.value.try_as_runtime_ref()?;
                    Some(UiWorkflowInputDescriptor {
                        id: input.id,
                        label: input.label.clone(),
                        param_type: value.param_type.clone(),
                        optional: value.optional,
                    })
                })
                .collect(),
        }
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

use std::process::Command;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::ErrorDetails;

use super::{
    ReturnedToolOutput, Tool, ToolInputDescriptor, ToolOutputAction, ToolParsedParamValue,
    ToolPresetParameterValue, ToolRuntimeInputDescriptor,
};

#[derive(Clone, Debug, Deserialize, specta::Type)]
pub struct NewUserDefinedTool {
    label: String,
    inputs: Vec<NewToolInput>,
    command: String,
    output_actions: ToolOutputAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct UserDefinedTool {
    pub label: String,
    pub inputs: Vec<ToolInputDescriptor>,
    pub command: String,
    pub output_actions: ToolOutputAction,
    pub id: Uuid,
    #[serde(default)]
    pub built_in: bool,
}

impl From<NewUserDefinedTool> for UserDefinedTool {
    fn from(value: NewUserDefinedTool) -> Self {
        Self {
            built_in: false,
            label: value.label,
            inputs: value.inputs.into_iter().map_into().collect(),
            command: value.command,
            output_actions: value.output_actions,
            id: Uuid::new_v4(),
        }
    }
}

impl Tool for UserDefinedTool {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_label(&self) -> String {
        self.label.clone()
    }

    fn get_expected_input_parameters(&self) -> Vec<ToolInputDescriptor> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> ToolOutputAction {
        self.output_actions.clone()
    }

    fn execute(
        &self,
        params: Vec<ToolParsedParamValue>,
    ) -> Result<Option<ReturnedToolOutput>, ErrorDetails> {
        let mut command = Command::new(&self.command);
        for param in params {
            command.arg(param.to_command_string());
        }
        command
            .output()
            .map_err(|err| ErrorDetails::IoError(err.to_string()))
            .map(|output| Some(ReturnedToolOutput::Command(output.into())))
    }

    fn dyn_clone(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn as_user_defined_tool(&self) -> Option<UserDefinedTool> {
        if self.built_in {
            None
        } else {
            Some(self.clone())
        }
    }
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(tag = "type")]
pub enum NewToolInput {
    Preset {
        value: ToolPresetParameterValue,
    },
    Runtime {
        label: String,
        param_type: ToolInputType,
        optional: bool,
    },
}

impl From<NewToolInput> for ToolInputDescriptor {
    fn from(value: NewToolInput) -> Self {
        match value {
            NewToolInput::Runtime {
                label,
                param_type,
                optional,
            } => Self::Runtime(ToolRuntimeInputDescriptor {
                label,
                param_type,
                optional,
                id: Uuid::new_v4(),
            }),
            NewToolInput::Preset { value } => Self::Preset(value),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, strum::EnumDiscriminants)]
#[serde(tag = "type", content = "options")]
#[strum_discriminants(derive(Serialize, Deserialize, specta::Type, strum::EnumIter))]
pub enum ToolInputType {
    Float,
    Int,
    String,
    Layer(LayerType),
    Dataset,
    Option(Vec<String>),
    /// bool to determine if this file is an output of the tool or not
    File(bool),
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, specta::Type, strum::EnumIter))]
pub enum LayerType {
    Vector,
    Raster,
    Any,
}

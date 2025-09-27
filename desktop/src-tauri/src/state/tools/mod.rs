pub mod user_defined;

use std::{path::PathBuf, process::Output as CommandOutput};
use tauri::AppHandle;
pub use user_defined::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::ErrorDetails, files::get_random_temp_path,
    tools::describe_landforms::DescribeLandformsTool, ui::ToolDescriptor,
};

use super::{
    dataset_collection::{DatasetCollection, NonEmptyDelegatorImpl},
    gis::combined::DatasetLayerIndex,
};

pub trait Tool: Send + Sync {
    fn get_id(&self) -> Uuid;
    fn get_label(&self) -> String;

    fn get_expected_input_parameters(&self) -> Vec<ToolInputDescriptor>;

    fn get_output_actions(&self) -> ToolOutputAction;

    fn parse_input_parameters(
        &self,
        params: Vec<ToolParameter>,
        project: &mut DatasetCollection,
        app: &AppHandle,
    ) -> Result<Vec<ToolParsedParamValue>, ErrorDetails> {
        let mut parsed = Vec::new();
        for expected in self.get_expected_input_parameters() {
            let parsed_val = match expected {
                ToolInputDescriptor::Preset(value) => ToolParsedParamValue::from_preset(value, app),
                ToolInputDescriptor::Runtime(ToolRuntimeInputDescriptor {
                    label,
                    param_type,
                    optional,
                    id,
                }) => {
                    let param = params.iter().find(|param| param.id == id);
                    if param.is_none() && optional {
                        continue;
                    };
                    let param = param.ok_or_else(|| {
                        ErrorDetails::Other(format!(
                            "Did not get required parameter {} for tool {}",
                            label,
                            self.get_label()
                        ))
                    })?;
                    ToolParsedParamValue::parse_from(param.value.clone(), param_type, project, app)?
                }
            };
            parsed.push(parsed_val);
        }
        Ok(parsed)
    }

    fn run(
        &self,
        params: Vec<ToolParameter>,
        project: &mut DatasetCollection,
        app: &AppHandle,
    ) -> Result<ToolOutput, ErrorDetails> {
        let mut output_files = Vec::new();
        let params = self.parse_input_parameters(params, project, app)?;

        for param in &params {
            if let Some(file) = param.try_as_file_ref()
                && file.use_as_output
            {
                output_files.push(file.path.clone())
            };
        }

        Ok(ToolOutput {
            returned_output: self.execute(params)?,
            files: output_files,
        })
    }

    fn execute(
        &self,
        params: Vec<ToolParsedParamValue>,
    ) -> Result<Option<ReturnedToolOutput>, ErrorDetails>;

    fn for_ui(&self) -> ToolDescriptor {
        ToolDescriptor {
            label: self.get_label(),
            inputs: self
                .get_expected_input_parameters()
                .into_iter()
                .filter_map(ToolInputDescriptor::try_as_runtime)
                .collect(),
            id: self.get_id(),
        }
    }

    fn dyn_clone(&self) -> Box<dyn Tool>;

    /// All but one implementation will return false so we might as well implement it here
    fn as_user_defined_tool(&self) -> Option<UserDefinedTool> {
        None
    }
}

impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolOutput {
    pub returned_output: Option<ReturnedToolOutput>,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "value")]
pub enum ReturnedToolOutput {
    Command(Output),
    String(String),
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: Option<i32>,
}

impl From<CommandOutput> for Output {
    fn from(value: CommandOutput) -> Self {
        Self {
            status: value.status.code(),
            stdout: String::from_utf8_lossy(&value.stdout).to_string(),
            stderr: String::from_utf8_lossy(&value.stderr).to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct SavedToolOutputAction {
    // The label of the tool that generated this output
    pub tool: String,
    pub output: ToolOutput,
    pub read: bool,
    pub id: uuid::Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolOutputAction {
    pub alert_output: bool,
    pub load_layers: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum ToolInputDescriptor {
    Preset(ToolPresetParameterValue),
    Runtime(ToolRuntimeInputDescriptor),
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ToolRuntimeInputDescriptor {
    pub label: String,
    pub param_type: ToolInputType,
    pub optional: bool,
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ToolParameter {
    pub id: Uuid,
    pub value: ToolParameterValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum ToolParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    Layer(DatasetLayerIndex),
    Dataset(usize),
    Option(String),
    Flag(bool),
    File(ToolFileInput),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    specta::Type,
    strum::EnumTryAs,
    strum::EnumDiscriminants,
)]
#[serde(tag = "type", content = "value")]
#[strum_discriminants(derive(Serialize, Deserialize, specta::Type, strum::EnumIter))]
pub enum ToolPresetParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    File(ToolFileInput),
}

impl ToolParsedParamValue {
    fn from_preset(value: ToolPresetParameterValue, app: &AppHandle) -> Self {
        match value {
            ToolPresetParameterValue::Float(num) => ToolParsedParamValue::Float(num),
            ToolPresetParameterValue::Int(num) => ToolParsedParamValue::Int(num),
            ToolPresetParameterValue::String(string) => ToolParsedParamValue::String(string),
            ToolPresetParameterValue::File(file) => {
                ToolParsedParamValue::File(ToolParsedFileParameter {
                    path: file.get_path(app),
                    use_as_output: false,
                })
            }
        }
    }
}

impl From<ToolPresetParameterValue> for ToolParameterValue {
    fn from(value: ToolPresetParameterValue) -> Self {
        match value {
            ToolPresetParameterValue::Float(num) => Self::Float(num),
            ToolPresetParameterValue::Int(num) => Self::Int(num),
            ToolPresetParameterValue::String(string) => Self::String(string),
            ToolPresetParameterValue::File(path) => Self::File(path),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    specta::Type,
    strum::EnumTryAs,
    strum::EnumDiscriminants,
)]
#[serde(tag = "type", content = "value")]
pub enum ToolFileInput {
    Temp(Option<String>),
    Named(PathBuf),
}

impl ToolFileInput {
    pub fn get_path(self, app: &AppHandle) -> PathBuf {
        match self {
            ToolFileInput::Named(path) => path,
            ToolFileInput::Temp(ext) => get_random_temp_path(app, ext),
        }
    }
}

#[derive(Debug, strum::EnumTryAs)]
pub enum ToolParsedParamValue {
    Float(f64),
    Int(i64),
    String(String),
    Option(String),
    File(ToolParsedFileParameter),
}

impl ToolParsedParamValue {
    pub fn to_command_string(&self) -> String {
        match self {
            ToolParsedParamValue::Float(num) => num.to_string(),
            ToolParsedParamValue::Int(num) => num.to_string(),
            ToolParsedParamValue::String(str) => str.to_string(),
            // TODO: Try see if we can return the layer index too
            ToolParsedParamValue::Option(string) => string.clone(),
            ToolParsedParamValue::File(file) => file.path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ToolParsedFileParameter {
    pub path: PathBuf,
    pub use_as_output: bool,
}

impl ToolParsedParamValue {
    pub fn parse_from(
        param: ToolParameterValue,
        expected: ToolInputType,
        datasets: &mut DatasetCollection,
        app: &AppHandle,
    ) -> Result<Self, ErrorDetails> {
        Ok(match (param, expected) {
            (ToolParameterValue::Float(num), ToolInputType::Float) => {
                ToolParsedParamValue::Float(num)
            }
            (ToolParameterValue::Int(num), ToolInputType::Int) => ToolParsedParamValue::Int(num),
            (ToolParameterValue::String(str), ToolInputType::String) => {
                ToolParsedParamValue::String(str)
            }
            (ToolParameterValue::Dataset(index), ToolInputType::Dataset) => {
                ToolParsedParamValue::File(ToolParsedFileParameter {
                    use_as_output: false,
                    path: datasets
                        .get_dataset(index)
                        .ok_or_else(|| ErrorDetails::Other("Missing dataset".to_string()))?
                        .dataset
                        .file_name
                        .clone(),
                })
            }
            (ToolParameterValue::Layer(index), ToolInputType::Layer(kind)) => match kind {
                LayerType::Vector => ToolParsedParamValue::File(ToolParsedFileParameter {
                    use_as_output: false,
                    path: datasets
                        .get(index)
                        .and_then(|layer| layer.try_as_vector())
                        .ok_or_else(|| ErrorDetails::Other("Missing vector layer".to_string()))?
                        .info
                        .shared
                        .name
                        .clone(),
                }),
                LayerType::Raster => ToolParsedParamValue::File(ToolParsedFileParameter {
                    use_as_output: false,
                    path: datasets
                        .get(index)
                        .and_then(|layer| layer.try_as_raster())
                        .ok_or_else(|| ErrorDetails::Other("Missing raster layer".to_string()))?
                        .info
                        .shared
                        .name
                        .clone(),
                }),
                LayerType::Any => ToolParsedParamValue::File(ToolParsedFileParameter {
                    use_as_output: false,
                    path: datasets
                        .get(index)
                        .ok_or_else(|| ErrorDetails::Other("Missing layer".to_string()))?
                        .shared_ref()
                        .name
                        .clone(),
                }),
            },
            (ToolParameterValue::File(file), ToolInputType::Layer(_kind)) => {
                ToolParsedParamValue::File(ToolParsedFileParameter {
                    use_as_output: false,
                    path: file.get_path(app),
                })
            }
            (ToolParameterValue::Option(option), ToolInputType::Option(options)) => {
                parse_option(option, options)?
            }
            (ToolParameterValue::File(file), ToolInputType::File(use_as_output)) => {
                ToolParsedParamValue::File(ToolParsedFileParameter {
                    path: file.get_path(app),
                    use_as_output,
                })
            }
            (got, expected) => Err(ErrorDetails::Other(format!(
                "Mismatched command types, expected {expected:?} but got {got:?}"
            )))?,
        })
    }
}

pub fn get_built_in_tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(DescribeLandformsTool)]
}

pub fn parse_option(
    option: String,
    options: Vec<String>,
) -> Result<ToolParsedParamValue, ErrorDetails> {
    if !options.contains(&option) {
        return Err(ErrorDetails::Other(
            "Somehow got unallowed option".to_string(),
        ));
    } else {
        Ok(ToolParsedParamValue::Option(option))
    }
}

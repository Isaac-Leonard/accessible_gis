use std::{
    mem::transmute,
    path::PathBuf,
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::ErrorDetails,
    gdal_if::{LayerIndex, LayerIndexDiscriminants},
    tools::describe_landforms::DescribeLandformsTool,
    ui::ToolDescriptor,
};

use super::{
    dataset_collection::NonEmptyDelegatorImpl,
    gis::{
        combined::{DatasetLayerIndex, RasterIndex, VectorIndex},
        dataset::StatefulDataset,
        raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
    projects::Project,
};

pub trait Tool: Send + Sync {
    fn get_label(&self) -> String;

    fn get_expected_input_parameters(&self) -> Vec<Input>;

    fn get_output_actions(&self) -> Vec<ToolOutputActionDiscriptor>;

    fn parse_input_parameters<'a>(
        &self,
        params: Vec<ParameterValue>,
        project: &'a mut Project,
    ) -> Result<Vec<NamedParsedParamValue<'a>>, ErrorDetails> {
        let mut parsed = Vec::new();
        let mut params = params.into_iter();
        for expected in self.get_expected_input_parameters() {
            let parsed_val = NamedParsedParamValue::parse_from(&mut params, expected, project)?;
            // TODO: This is really bad
            // This needs to be refactored properly however I suspect that cannot be done without rewriting large parts of the gdal-rs library.
            // This shouldn't currently cause any issues as all of this code is currently affectively single threaded for now.
            let parsed_val = unsafe {
                parsed_val
                    .map(|v| transmute::<NamedParsedParamValue<'_>, NamedParsedParamValue<'a>>(v))
            };
            match parsed_val {
                Some(val) => parsed.push(val),
                _ => eprintln!("Got none when parsing params"),
            }
        }
        if params.count() != 0 {
            Err(ErrorDetails::Other(
                "Not all params used for tool call".to_string(),
            ))
        } else {
            Ok(parsed)
        }
    }

    fn run(&self, params: Vec<ParameterValue>, project: &mut Project) -> Result<(), ErrorDetails> {
        #[derive(Debug)]
        pub enum ToolOutputActionDiscriptorRequiresProject {
            AlertOutput,
            File(PathBuf),
        }
        let mut saved_actions = Vec::new();
        let mut requires_project_actions = Vec::new();
        let data = {
            let params = self.parse_input_parameters(params, project)?;
            let actions = self.get_output_actions();
            for action in actions {
                match action {
                    ToolOutputActionDiscriptor::Alert(msg) => {
                        saved_actions.push(SavedToolOutputAction {
                            tool: self.get_label(),
                            message: msg,
                            read: false,
                            id: uuid::Uuid::new_v4(),
                        })
                    }
                    ToolOutputActionDiscriptor::LoadAsDataset(index) => {
                        println!["{index}: {params:?}"];
                        let file = params[index]
                            .try_as_raw_ref()
                            .unwrap_or_else(|| params[index].try_as_named_ref().unwrap().1)
                            .try_as_file_ref()
                            .unwrap()
                            .clone();
                        requires_project_actions
                            .push(ToolOutputActionDiscriptorRequiresProject::File(file))
                    }
                    ToolOutputActionDiscriptor::AlertOutput => requires_project_actions
                        .push(ToolOutputActionDiscriptorRequiresProject::AlertOutput),
                };
            }
            self.execute(&params)?
        };

        for action in requires_project_actions {
            match action {
                ToolOutputActionDiscriptorRequiresProject::AlertOutput => {
                    let output = data.as_ref().ok_or_else(|| {
                        ErrorDetails::Other("This tool should return data but doesn't".to_string())
                    })?;
                    let output = match output {
                        ToolOutput::Command(output) => {
                            "Stdout:\n".to_string()
                                + &String::from_utf8_lossy(&output.stdout)
                                + "\nStderr:\n"
                                + &String::from_utf8_lossy(&output.stderr)
                        }
                        ToolOutput::String(string) => string.clone(),
                    };
                    saved_actions.push(SavedToolOutputAction {
                        tool: self.get_label(),
                        message: output,
                        read: false,
                        id: uuid::Uuid::new_v4(),
                    })
                }
                ToolOutputActionDiscriptorRequiresProject::File(path) => {
                    project
                        .datasets
                        .open(path, &project.settings)
                        .map_err(ErrorDetails::OpenDatasetError)?;
                }
            }
        }
        project.tool_outputs.extend(saved_actions);
        Ok(())
    }

    fn execute(&self, params: &[NamedParsedParamValue])
    -> Result<Option<ToolOutput>, ErrorDetails>;

    fn for_ui(&self) -> ToolDescriptor {
        ToolDescriptor {
            label: self.get_label(),
            inputs: self.get_expected_input_parameters(),
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

pub enum ToolOutput {
    Command(Output),
    String(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct SavedToolOutputAction {
    // The label of the tool that generated this output
    pub tool: String,
    pub message: String,
    pub read: bool,
    pub id: uuid::Uuid,
}

#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type, strum::EnumDiscriminants,
)]
#[serde(tag = "type", content = "value")]
#[strum_discriminants(derive(Serialize, Deserialize, specta::Type, strum::EnumIter))]
pub enum ToolOutputActionDiscriptor {
    Alert(String),
    LoadAsDataset(usize),
    AlertOutput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct UserDefinedTool {
    label: String,
    inputs: Vec<Input>,
    command: String,
    output_actions: Vec<ToolOutputActionDiscriptor>,
}

impl Tool for UserDefinedTool {
    fn get_label(&self) -> String {
        self.label.clone()
    }

    fn get_expected_input_parameters(&self) -> Vec<Input> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> Vec<ToolOutputActionDiscriptor> {
        self.output_actions.clone()
    }

    fn execute(
        &self,
        params: &[NamedParsedParamValue],
    ) -> Result<Option<ToolOutput>, ErrorDetails> {
        let mut command = Command::new(&self.command);
        for param in params {
            match param {
                NamedParsedParamValue::Named(name, value) => {
                    command.arg(name).arg(value.to_command_string())
                }
                NamedParsedParamValue::Raw(value) => command.arg(value.to_command_string()),
                NamedParsedParamValue::Flag(flag) => command.arg(flag),
            };
        }
        command
            .output()
            .map_err(|err| ErrorDetails::IoError(err.to_string()))
            .map(|output| Some(ToolOutput::Command(output)))
    }

    fn dyn_clone(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn as_user_defined_tool(&self) -> Option<UserDefinedTool> {
        Some(self.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Input {
    pub label: String,
    pub name: Option<String>,
    pub param_type: InputType,
}

#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type, strum::EnumDiscriminants,
)]
#[serde(tag = "type", content = "options")]
#[strum_discriminants(derive(Serialize, Deserialize, specta::Type, strum::EnumIter))]
pub enum InputType {
    Float,
    Int,
    String,
    Layer(LayerIndexDiscriminants),
    Dataset,
    Option(Vec<String>),
    Flag,
    File,
    Preset(PresetParameterValue),
}

#[derive(Clone, Debug, Deserialize, specta::Type, strum::EnumTryAs)]
#[serde(tag = "type", content = "value")]
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    Layer(DatasetLayerIndex),
    Dataset(usize),
    Option(String),
    Flag(bool),
    File(PathBuf),
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
pub enum PresetParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    File(PathBuf),
}

#[derive(Debug, strum::EnumTryAs)]
pub enum ParsedParamValue<'a> {
    Float(f64),
    Int(i64),
    String(String),
    Vector(StatefulVectorLayer<'a>),
    Raster(StatefulRasterBand<'a>),
    Dataset(&'a StatefulDataset),
    Option(String),
    File(PathBuf),
}

impl ParsedParamValue<'_> {
    pub fn to_command_string(&self) -> String {
        match self {
            ParsedParamValue::Float(num) => num.to_string(),
            ParsedParamValue::Int(num) => num.to_string(),
            ParsedParamValue::String(str) => str.to_string(),
            // TODO: Try see if we can return the layer index too
            ParsedParamValue::Vector(layer) => layer.info.shared.name.to_string_lossy().to_string(),
            // TODO: Try see if we can return the band index too
            ParsedParamValue::Raster(band) => band.info.shared.name.to_string_lossy().to_string(),
            ParsedParamValue::Dataset(dataset) => {
                dataset.dataset.file_name.to_string_lossy().to_string()
            }
            ParsedParamValue::Option(string) => string.clone(),
            ParsedParamValue::File(path) => path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug, strum::EnumTryAs)]
pub enum NamedParsedParamValue<'a> {
    Named(String, ParsedParamValue<'a>),
    Raw(ParsedParamValue<'a>),
    Flag(String),
}

impl<'a> NamedParsedParamValue<'a> {
    pub fn parse_from(
        params: &mut impl Iterator<Item = ParameterValue>,
        expected: Input,
        project: &'a mut Project,
    ) -> Result<Option<Self>, ErrorDetails> {
        let val: ParsedParamValue = match expected.param_type {
            InputType::Preset(value) => match value {
                PresetParameterValue::Float(num) => ParsedParamValue::Float(num),
                PresetParameterValue::Int(num) => ParsedParamValue::Int(num),
                PresetParameterValue::String(string) => ParsedParamValue::String(string),
                PresetParameterValue::File(path) => ParsedParamValue::File(path),
            },
            expected_type => {
                let Some(value) = params.next() else {
                    return Err(ErrorDetails::Other(
                        "Not enough params passed for tool".to_string(),
                    ));
                };

                match (value, expected_type) {
                    (ParameterValue::Float(num), InputType::Float) => ParsedParamValue::Float(num),
                    (ParameterValue::Int(num), InputType::Int) => ParsedParamValue::Int(num),
                    (ParameterValue::String(str), InputType::String) => {
                        ParsedParamValue::String(str)
                    }
                    (ParameterValue::Dataset(index), InputType::Dataset) => {
                        ParsedParamValue::Dataset(
                            project.datasets.get_dataset(index).ok_or_else(|| {
                                ErrorDetails::Other("Missing dataset".to_string())
                            })?,
                        )
                    }
                    (ParameterValue::Layer(index), InputType::Layer(kind)) => {
                        match (index.layer, kind) {
                            (LayerIndex::Vector(layer_index), LayerIndexDiscriminants::Vector) => {
                                ParsedParamValue::Vector(
                                    project
                                        .get_vector(VectorIndex {
                                            dataset: index.dataset,
                                            layer: layer_index,
                                        })
                                        .ok_or_else(|| {
                                            ErrorDetails::Other("Missing vector layer".to_string())
                                        })?,
                                )
                            }
                            (LayerIndex::Raster(band_index), LayerIndexDiscriminants::Raster) => {
                                ParsedParamValue::Raster(
                                    project
                                        .get_raster(RasterIndex {
                                            dataset: index.dataset,
                                            band: band_index,
                                        })
                                        .ok_or_else(|| {
                                            ErrorDetails::Other("Missing raster layer".to_string())
                                        })?,
                                )
                            }
                            _ => Err(ErrorDetails::Other("Mismatched layer types".to_string()))?,
                        }
                    }
                    (ParameterValue::Option(option), InputType::Option(options)) => {
                        if !options.contains(&option) {
                            return Err(ErrorDetails::Other(
                                "Somehow got unallowed option".to_string(),
                            ));
                        } else {
                            ParsedParamValue::Option(option)
                        }
                    }
                    (ParameterValue::Flag(include), InputType::Flag) => {
                        return Ok(include.then_some(Self::Flag(expected.name.ok_or_else(
                            || ErrorDetails::Other("Missing name for flag".to_string()),
                        )?)));
                    }
                    (ParameterValue::File(file), InputType::File) => ParsedParamValue::File(file),
                    _ => Err(ErrorDetails::Other("Mismatched command types".to_string()))?,
                }
            }
        };

        Ok(Some(match expected.name {
            Some(name) => Self::Named(name, val),
            None => Self::Raw(val),
        }))
    }
}

pub fn get_built_in_tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(DescribeLandformsTool)]
}

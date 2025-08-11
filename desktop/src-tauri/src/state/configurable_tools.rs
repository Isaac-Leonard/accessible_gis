use std::{
    mem::transmute,
    path::PathBuf,
    process::{Command, Output as CommandOutput},
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::ErrorDetails,
    gdal_if::{LayerIndex, LayerIndexDiscriminants},
    tools::describe_landforms::DescribeLandformsTool,
    ui::ToolDescriptor,
};

use super::{
    dataset_collection::{DatasetCollection, NonEmptyDelegatorImpl},
    gis::{
        combined::{DatasetLayerIndex, RasterIndex, VectorIndex},
        dataset::StatefulDataset,
        raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ToolOutput {
    pub returned_output: Option<ReturnedToolOutput>,
    pub files: Vec<PathBuf>,
}

pub trait Tool: Send + Sync {
    fn get_label(&self) -> String;

    fn get_expected_input_parameters(&self) -> Vec<Input>;

    fn get_output_actions(&self) -> ToolOutputAction;

    fn parse_input_parameters<'a>(
        &self,
        params: Vec<ParameterValue>,
        project: &'a mut DatasetCollection,
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

    fn run(
        &self,
        params: Vec<ParameterValue>,
        project: &mut DatasetCollection,
    ) -> Result<ToolOutput, ErrorDetails> {
        let mut output_files = Vec::new();
        let params = self.parse_input_parameters(params, project)?;

        for param in &params {
            let param_val = param
                .try_as_raw_ref()
                .unwrap_or_else(|| param.try_as_named_ref().unwrap().1);
            if let Some(file) = param_val.try_as_file_ref()
                && file.use_as_output
            {
                output_files.push(file.path.clone())
            };
        }

        Ok(ToolOutput {
            returned_output: self.execute(&params)?,
            files: output_files,
        })
    }

    fn execute(
        &self,
        params: &[NamedParsedParamValue],
    ) -> Result<Option<ReturnedToolOutput>, ErrorDetails>;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct UserDefinedTool {
    label: String,
    inputs: Vec<Input>,
    command: String,
    output_actions: ToolOutputAction,
}

impl Tool for UserDefinedTool {
    fn get_label(&self) -> String {
        self.label.clone()
    }

    fn get_expected_input_parameters(&self) -> Vec<Input> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> ToolOutputAction {
        self.output_actions.clone()
    }

    fn execute(
        &self,
        params: &[NamedParsedParamValue],
    ) -> Result<Option<ReturnedToolOutput>, ErrorDetails> {
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
            .map(|output| Some(ReturnedToolOutput::Command(output.into())))
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
    File(bool),
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
    File(ParsedFileParameter),
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
            ParsedParamValue::File(file) => file.path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ParsedFileParameter {
    path: PathBuf,
    use_as_output: bool,
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
        datasets: &'a mut DatasetCollection,
    ) -> Result<Option<Self>, ErrorDetails> {
        let val: ParsedParamValue = match expected.param_type {
            InputType::Preset(value) => match value {
                PresetParameterValue::Float(num) => ParsedParamValue::Float(num),
                PresetParameterValue::Int(num) => ParsedParamValue::Int(num),
                PresetParameterValue::String(string) => ParsedParamValue::String(string),
                PresetParameterValue::File(path) => ParsedParamValue::File(ParsedFileParameter {
                    path,
                    use_as_output: false,
                }),
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
                            datasets.get_dataset(index).ok_or_else(|| {
                                ErrorDetails::Other("Missing dataset".to_string())
                            })?,
                        )
                    }
                    (ParameterValue::Layer(index), InputType::Layer(kind)) => {
                        match (index.layer, kind) {
                            (LayerIndex::Vector(layer_index), LayerIndexDiscriminants::Vector) => {
                                ParsedParamValue::Vector(
                                    datasets
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
                                    datasets
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
                    (ParameterValue::File(path), InputType::File(use_as_output)) => {
                        ParsedParamValue::File(ParsedFileParameter {
                            path,
                            use_as_output,
                        })
                    }
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

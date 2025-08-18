pub mod user_defined;

use std::{mem::transmute, path::PathBuf, process::Output as CommandOutput};
pub use user_defined::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::ErrorDetails, gdal_if::LayerIndexDiscriminants,
    tools::describe_landforms::DescribeLandformsTool, ui::ToolDescriptor,
};

use super::{
    dataset_collection::{DatasetCollection, NonEmptyDelegatorImpl},
    gis::{
        combined::DatasetLayerIndex, dataset::StatefulDataset, raster::StatefulRasterBand,
        vector::StatefulVectorLayer,
    },
};

pub trait Tool: Send + Sync {
    fn get_id(&self) -> Uuid;
    fn get_label(&self) -> String;

    fn get_expected_input_parameters(&self) -> Vec<ToolInputDescriptor>;

    fn get_output_actions(&self) -> ToolOutputAction;

    fn parse_input_parameters<'a>(
        &self,
        params: Vec<ToolParameter>,
        project: &'a mut DatasetCollection,
    ) -> Result<Vec<ToolNamedParsedParamValue<'a>>, ErrorDetails> {
        let mut parsed = Vec::new();
        let mut params = params.into_iter().map(|param| param.value);
        for expected in self.get_expected_input_parameters() {
            let parsed_val = ToolNamedParsedParamValue::parse_from(&mut params, expected, project)?;
            // TODO: This is really bad
            // This needs to be refactored properly however I suspect that cannot be done without rewriting large parts of the gdal-rs library.
            // This shouldn't currently cause any issues as all of this code is currently affectively single threaded for now.
            let parsed_val = unsafe {
                parsed_val.map(|v| {
                    transmute::<ToolNamedParsedParamValue<'_>, ToolNamedParsedParamValue<'a>>(v)
                })
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
        params: Vec<ToolParameter>,
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
        params: &[ToolNamedParsedParamValue],
    ) -> Result<Option<ReturnedToolOutput>, ErrorDetails>;

    fn for_ui(&self) -> ToolDescriptor {
        ToolDescriptor {
            label: self.get_label(),
            inputs: self.get_expected_input_parameters(),
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

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ToolInputDescriptor {
    pub label: String,
    pub name: Option<String>,
    pub param_type: ToolInputType,
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ToolParameter {
    id: Uuid,
    value: ToolParameterValue,
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
pub enum ToolPresetParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    File(PathBuf),
}

impl From<ToolPresetParameterValue> for ToolParsedParamValue<'_> {
    fn from(value: ToolPresetParameterValue) -> Self {
        match value {
            ToolPresetParameterValue::Float(num) => ToolParsedParamValue::Float(num),
            ToolPresetParameterValue::Int(num) => ToolParsedParamValue::Int(num),
            ToolPresetParameterValue::String(string) => ToolParsedParamValue::String(string),
            ToolPresetParameterValue::File(path) => {
                ToolParsedParamValue::File(ToolParsedFileParameter {
                    path,
                    use_as_output: false,
                })
            }
        }
    }
}

#[derive(Debug, strum::EnumTryAs)]
pub enum ToolParsedParamValue<'a> {
    Float(f64),
    Int(i64),
    String(String),
    Vector(StatefulVectorLayer<'a>),
    Raster(StatefulRasterBand<'a>),
    Dataset(&'a StatefulDataset),
    Option(String),
    File(ToolParsedFileParameter),
}

impl ToolParsedParamValue<'_> {
    pub fn to_command_string(&self) -> String {
        match self {
            ToolParsedParamValue::Float(num) => num.to_string(),
            ToolParsedParamValue::Int(num) => num.to_string(),
            ToolParsedParamValue::String(str) => str.to_string(),
            // TODO: Try see if we can return the layer index too
            ToolParsedParamValue::Vector(layer) => {
                layer.info.shared.name.to_string_lossy().to_string()
            }
            // TODO: Try see if we can return the band index too
            ToolParsedParamValue::Raster(band) => {
                band.info.shared.name.to_string_lossy().to_string()
            }
            ToolParsedParamValue::Dataset(dataset) => {
                dataset.dataset.file_name.to_string_lossy().to_string()
            }
            ToolParsedParamValue::Option(string) => string.clone(),
            ToolParsedParamValue::File(file) => file.path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ToolParsedFileParameter {
    path: PathBuf,
    use_as_output: bool,
}

#[derive(Debug, strum::EnumTryAs)]
pub enum ToolNamedParsedParamValue<'a> {
    Named(String, ToolParsedParamValue<'a>),
    Raw(ToolParsedParamValue<'a>),
    Flag(String),
}

impl<'a> ToolNamedParsedParamValue<'a> {
    pub fn parse_from(
        params: &mut impl Iterator<Item = ToolParameterValue>,
        expected: ToolInputDescriptor,
        datasets: &'a mut DatasetCollection,
    ) -> Result<Option<Self>, ErrorDetails> {
        let val: ToolParsedParamValue = match expected.param_type {
            ToolInputType::Preset(value) => ToolParsedParamValue::from(value),
            expected_type => {
                let Some(value) = params.next() else {
                    return Err(ErrorDetails::Other(
                        "Not enough params passed for tool".to_string(),
                    ));
                };

                match (value, expected_type) {
                    (ToolParameterValue::Float(num), ToolInputType::Float) => {
                        ToolParsedParamValue::Float(num)
                    }
                    (ToolParameterValue::Int(num), ToolInputType::Int) => {
                        ToolParsedParamValue::Int(num)
                    }
                    (ToolParameterValue::String(str), ToolInputType::String) => {
                        ToolParsedParamValue::String(str)
                    }
                    (ToolParameterValue::Dataset(index), ToolInputType::Dataset) => {
                        ToolParsedParamValue::Dataset(
                            datasets.get_dataset(index).ok_or_else(|| {
                                ErrorDetails::Other("Missing dataset".to_string())
                            })?,
                        )
                    }
                    (ToolParameterValue::Layer(index), ToolInputType::Layer(kind)) => match kind {
                        LayerIndexDiscriminants::Vector => ToolParsedParamValue::Vector(
                            datasets
                                .get(index)
                                .and_then(|layer| layer.try_as_vector())
                                .ok_or_else(|| {
                                    ErrorDetails::Other("Missing vector layer".to_string())
                                })?,
                        ),
                        LayerIndexDiscriminants::Raster => ToolParsedParamValue::Raster(
                            datasets
                                .get(index)
                                .and_then(|layer| layer.try_as_raster())
                                .ok_or_else(|| {
                                    ErrorDetails::Other("Missing raster layer".to_string())
                                })?,
                        ),
                    },
                    (ToolParameterValue::Option(option), ToolInputType::Option(options)) => {
                        parse_option(option, options)?
                    }
                    (ToolParameterValue::Flag(include), ToolInputType::Flag) => {
                        return Ok(include.then_some(Self::Flag(expected.name.ok_or_else(
                            || ErrorDetails::Other("Missing name for flag".to_string()),
                        )?)));
                    }
                    (ToolParameterValue::File(path), ToolInputType::File(use_as_output)) => {
                        ToolParsedParamValue::File(ToolParsedFileParameter {
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

pub fn parse_option(
    option: String,
    options: Vec<String>,
) -> Result<ToolParsedParamValue<'static>, ErrorDetails> {
    if !options.contains(&option) {
        return Err(ErrorDetails::Other(
            "Somehow got unallowed option".to_string(),
        ));
    } else {
        Ok(ToolParsedParamValue::Option(option))
    }
}

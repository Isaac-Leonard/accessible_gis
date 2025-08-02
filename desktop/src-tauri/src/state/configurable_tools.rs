use std::{mem::transmute, process::Command};

use serde::{Deserialize, Serialize};

use crate::{
    errors::ErrorDetails,
    gdal_if::{LayerIndex, LayerIndexDiscriminants},
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

    fn parse_input_parameters<'a>(
        &self,
        params: Vec<ParameterValue>,
        project: &'a mut Project,
    ) -> Result<Vec<NamedParsedParamValue<'a>>, ErrorDetails> {
        let expected_inputs = self.get_expected_input_parameters();
        if expected_inputs.len() != params.len() {
            return Err(ErrorDetails::Other(format!(
                "Mismatched parameters, got unexpected number of parameters for tool {}",
                self.get_label()
            )));
        }

        let mut parsed = Vec::new();

        for (expected, got) in expected_inputs.into_iter().zip(params) {
            let parsed_val = NamedParsedParamValue::parse_from(got, expected, project)?;
            // TODO: This is really bad
            // This needs to be refactored properly however I suspect that cannot be done without rewriting large parts of the gdal-rs library.
            // This shouldn't currently cause any issues as all of this code is currently affectively single threaded for now.
            let parsed_val = unsafe {
                parsed_val
                    .map(|v| transmute::<NamedParsedParamValue<'_>, NamedParsedParamValue<'a>>(v))
            };
            match parsed_val {
                Some(val) => parsed.push(val),
                _ => {}
            }
        }

        Ok(parsed)
    }

    fn run(&self, params: Vec<ParameterValue>, project: &mut Project) -> Result<(), ErrorDetails> {
        let params = self.parse_input_parameters(params, project)?;
        let action = SavedToolOutputAction {
            tool: self.get_label(),
            action: self.execute(params)?,
            read: false,
            id: uuid::Uuid::new_v4(),
        };
        project.tool_outputs.push(action);
        Ok(())
    }

    fn execute(&self, params: Vec<NamedParsedParamValue>)
    -> Result<ToolOutputAction, ErrorDetails>;

    fn for_ui(&self) -> ToolDescriptor {
        ToolDescriptor {
            label: self.get_label(),
            inputs: self.get_expected_input_parameters(),
        }
    }

    fn dyn_clone(&self) -> Box<dyn Tool>;
}

impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "data")]
pub enum ToolOutputAction {
    Alert(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct SavedToolOutputAction {
    // The label of the tool that generated this output
    pub tool: String,
    pub action: ToolOutputAction,
    pub read: bool,
    pub id: uuid::Uuid,
}

#[derive(Clone, Debug)]
pub struct UserDefinedTool {
    label: String,
    inputs: Vec<Input>,
    command: String,
}

impl Tool for UserDefinedTool {
    fn get_label(&self) -> String {
        self.label.clone()
    }

    fn get_expected_input_parameters(&self) -> Vec<Input> {
        self.inputs.clone()
    }

    fn execute(
        &self,
        params: Vec<NamedParsedParamValue>,
    ) -> Result<ToolOutputAction, ErrorDetails> {
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
            .map_err(|err| ErrorDetails::IoError(err.to_string()));
        Ok(ToolOutputAction::Alert("Done".to_string()))
    }

    fn dyn_clone(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Input {
    pub label: String,
    pub name: Option<String>,
    pub param_type: InputType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "options")]
pub enum InputType {
    Float,
    Int,
    String,
    Layer(LayerIndexDiscriminants),
    Dataset,
    Option(Vec<String>),
    Flag,
}

#[derive(Clone, Debug, Deserialize, specta::Type, strum::EnumTryAs)]
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    Layer(DatasetLayerIndex),
    Dataset(usize),
    Option(String),
    Flag(bool),
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
        value: ParameterValue,
        expected: Input,
        project: &'a mut Project,
    ) -> Result<Option<Self>, ErrorDetails> {
        let val: ParsedParamValue = match (value, expected.param_type) {
            (ParameterValue::Float(num), InputType::Float) => ParsedParamValue::Float(num),
            (ParameterValue::Int(num), InputType::Int) => ParsedParamValue::Int(num),
            (ParameterValue::String(str), InputType::String) => ParsedParamValue::String(str),
            (ParameterValue::Dataset(index), InputType::Dataset) => ParsedParamValue::Dataset(
                project
                    .datasets
                    .get_dataset(index)
                    .ok_or_else(|| ErrorDetails::Other("Missing dataset".to_string()))?,
            ),
            (ParameterValue::Layer(index), InputType::Layer(kind)) => match (index.layer, kind) {
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
            },
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
                return Ok(include.then_some(Self::Flag(
                    expected
                        .name
                        .ok_or_else(|| ErrorDetails::Other("Missing name for flag".to_string()))?,
                )));
            }
            _ => Err(ErrorDetails::Other("Mismatched command types".to_string()))?,
        };

        Ok(Some(match expected.name {
            Some(name) => Self::Named(name, val),
            None => Self::Raw(val),
        }))
    }
}

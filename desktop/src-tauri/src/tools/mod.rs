pub mod describe_landforms;
pub mod shape_analysis;

use std::{
    path::Path,
    process::{Command, Output},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, path::BaseDirectory};
use uuid::Uuid;

use crate::state::tools::{
    LayerType, ToolInputDescriptor, ToolInputType, ToolOutputAction, ToolPresetParameterValue,
    ToolRuntimeInputDescriptor, UserDefinedTool,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum DemClassificationError {
    Geomorphons(String),
    Polygonise(String),
    MajorityFilter(String),
    LabelLandForms(String),
    FailToRun(String),
}

pub fn dem_to_landform_polygons(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    search: usize,
    threshold: f64,
    distance: usize,
    filter: usize,
    app: AppHandle,
) -> Result<(), DemClassificationError> {
    let path = app.path();
    let geomorphons_output = path
        .resolve("output_geomorphons.tif", BaseDirectory::Temp)
        .expect("Could not find temp directory");
    proc_to_result(geomorphons(
        input,
        &geomorphons_output,
        search,
        threshold,
        distance,
    )?)
    .map_err(DemClassificationError::Geomorphons)?;
    let majority_filter_output = path
        .resolve("output_majority_filter.tif", BaseDirectory::Temp)
        .expect("Could not find temp directory");
    proc_to_result(majority_filter(
        &geomorphons_output,
        &majority_filter_output,
        filter,
    )?)
    .map_err(DemClassificationError::MajorityFilter)?;
    proc_to_result(polygonise(&majority_filter_output, output.as_ref())?)
        .map_err(DemClassificationError::Polygonise)?;
    proc_to_result(label_landforms(output, app)?)
        .map_err(DemClassificationError::LabelLandForms)?;
    Ok(())
}

fn geomorphons(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    search: usize,
    threshold: f64,
    distance: usize,
) -> Result<Output, DemClassificationError> {
    let mut command = wbt();
    command.args(["-r", "Geomorphons", "-v"]);
    command.arg("--dem").arg(input.as_ref());
    command.arg("-o").arg(output.as_ref());
    command.arg("--search").arg(search.to_string().as_str());
    command
        .arg("--threshold")
        .arg(threshold.to_string().as_str());
    command.arg("--tdist").arg(distance.to_string().as_str());
    command.arg("--forms");
    command.arg("--cd").arg("./");
    run_program(command)
}

fn majority_filter(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    filter: usize,
) -> Result<Output, DemClassificationError> {
    let mut command = wbt();
    command.args(["-r", "MajorityFilter"]);
    command.arg("-i").arg(input.as_ref());
    command.arg("-o").arg(output.as_ref());
    command.arg("--filter").arg(filter.to_string().as_str());
    run_program(command)
}

fn polygonise(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<Output, DemClassificationError> {
    let mut command = Command::new("gdal_polygonize");
    command.arg(input.as_ref()).arg(output.as_ref());
    run_program(command)
}

fn label_landforms(
    input: impl AsRef<Path>,
    app: AppHandle,
) -> Result<Output, DemClassificationError> {
    let path = app.path();
    let mut command = Command::new("python3.13");
    command
        .arg(
            path.resolve("scripts/label_geomorphons.py", BaseDirectory::Resource)
                .unwrap(),
        )
        .arg(input.as_ref())
        .arg("dn");
    run_program(command)
}

fn wbt() -> Command {
    Command::new("whitebox_tools")
}

pub fn get_sieve_filter_tool() -> UserDefinedTool {
    static TOOL: LazyLock<UserDefinedTool> = LazyLock::new(|| UserDefinedTool {
        built_in: true,
        label: "Sieve filter".to_string(),
        inputs: vec![
            ToolInputDescriptor::Preset(ToolPresetParameterValue::String("-st".to_string())),
            ToolInputDescriptor::Runtime(ToolRuntimeInputDescriptor {
                label: "Filter threshold".to_string(),
                param_type: ToolInputType::Int,
                id: Uuid::new_v4(),
                optional: false,
            }),
            ToolInputDescriptor::Runtime(ToolRuntimeInputDescriptor {
                label: "Use 8 connectedness".to_string(),
                param_type: ToolInputType::Option(vec!["-4".to_string(), "-8".to_string()]),
                optional: true,
                id: Uuid::new_v4(),
            }),
            ToolInputDescriptor::Runtime(ToolRuntimeInputDescriptor {
                label: "Input".to_string(),
                param_type: ToolInputType::Layer(LayerType::Raster),
                optional: false,
                id: Uuid::new_v4(),
            }),
            ToolInputDescriptor::Runtime(ToolRuntimeInputDescriptor {
                label: "Output raster".to_string(),
                param_type: ToolInputType::File(true),
                optional: false,
                id: Uuid::new_v4(),
            }),
        ],
        command: "gdal_sieve.py".to_string(),
        output_actions: ToolOutputAction {
            alert_output: false,
            load_layers: vec![0],
        },
        id: Uuid::new_v4(),
    });
    TOOL.clone()
}

fn proc_to_result(output: Output) -> Result<String, String> {
    if !output.status.success() {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

fn run_program(mut command: Command) -> Result<Output, DemClassificationError> {
    command.output().map_err(|_| {
        DemClassificationError::FailToRun(command.get_program().to_string_lossy().to_string())
    })
}

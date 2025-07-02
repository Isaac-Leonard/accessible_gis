use std::{
    path::PathBuf,
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, path::BaseDirectory};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum DemClassificationError {
    Geomorphons(String),
    Polygonise(String),
    MajorityFilter(String),
    LabelLandForms(String),
}

pub fn dem_to_landform_polygons(
    input: PathBuf,
    search: usize,
    threshold: f64,
    distance: usize,
    filter: usize,
    app: AppHandle,
) -> Result<PathBuf, DemClassificationError> {
    let path = app.path();
    let geomorphons_output = path
        .resolve("output_geomorphons.tif", BaseDirectory::Temp)
        .unwrap();
    proc_to_result(geomorphons(
        input,
        &geomorphons_output,
        search,
        threshold,
        distance,
    ))
    .map_err(DemClassificationError::Geomorphons)?;
    let majority_filter_output = path
        .resolve("output_majority_filter.tif", BaseDirectory::Temp)
        .unwrap();
    proc_to_result(majority_filter(
        &geomorphons_output,
        &majority_filter_output,
        filter,
    ))
    .map_err(DemClassificationError::MajorityFilter)?;
    proc_to_result(polygonise(&majority_filter_output, output))
        .map_err(DemClassificationError::Polygonise)?;
    proc_to_result(label_landforms(output, app)).map_err(DemClassificationError::LabelLandForms)?;
    Ok(())
}

fn geomorphons(
    input: &PathBuf,
    output: &PathBuf,
    search: usize,
    threshold: f64,
    distance: usize,
) -> Output {
    let mut command = wbt();
    command.args(["-r", "Geomorphons", "-v"]);
    command.arg("--dem").arg(input);
    command.arg("-o").arg(output);
    command.arg("--search").arg(search.to_string().as_str());
    command
        .arg("--threshold")
        .arg(threshold.to_string().as_str());
    command.arg("--tdist").arg(distance.to_string().as_str());
    command.arg("--forms");
    command.arg("--cd").arg("./");
    command.output().unwrap()
}

fn majority_filter(input: &PathBuf, output: &PathBuf, filter: usize) -> Output {
    let mut command = wbt();
    command.args(["-r", "MajorityFilter"]);
    command.arg("-i").arg(input);
    command.arg("-o").arg(output);
    command.arg("--filter").arg(filter.to_string().as_str());
    command.output().unwrap()
}

fn polygonise(input: &PathBuf, output: &PathBuf) -> Output {
    let mut command = Command::new("gdal_polygonize");
    command.arg(input).arg(output);
    command.output().unwrap()
}

fn label_landforms(input: &PathBuf, app: AppHandle) -> Output {
    let path = app.path();
    let mut command = Command::new("python3.13");
    command
        .arg(
            path.resolve("scripts/label_geomorphons.py", BaseDirectory::Resource)
                .unwrap(),
        )
        .arg(input)
        .arg("dn");
    command.output().unwrap()
}

fn wbt() -> Command {
    Command::new("whitebox_tools")
}

fn proc_to_result(output: Output) -> Result<String, String> {
    if !output.status.success() {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

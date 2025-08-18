use gdal::spatial_ref::SpatialRef;
use uuid::Uuid;

use crate::{
    errors::ErrorDetails,
    gdal_if::Srs,
    state::{
        AppState,
        gis::combined::StatefulLayerEnum,
        tools::{NewUserDefinedTool, SavedToolOutputAction, ToolParameter, UserDefinedTool},
        workflows::{NewWorkflow, RuntimeInputs},
    },
};

#[tauri::command]
#[specta::specta]
pub fn reproject_layer(srs: Srs, name: &str, state: AppState) {
    state.with_current_dataset_mut_fallible(|ds, _| -> Result<(), ErrorDetails> {
        let layer = ds.get_current_layer();
        Ok(match layer {
            Some(StatefulLayerEnum::Vector(layer)) => {
                let output = layer
                    .reproject(name, srs)
                    .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
                eprint!("{:?}", output)
            }
            Some(StatefulLayerEnum::Raster(band)) => {
                let output = band
                    .reproject(name, srs)
                    .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
                eprint!("{:?}", output)
            }
            None => Err(ErrorDetails::Other(
                "No layer available to reproject".to_string(),
            ))?,
        })
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_srs(srs: Srs, state: AppState) {
    let srs = match srs {
        Srs::Proj(proj_string) => SpatialRef::from_proj4(&proj_string),
        Srs::Wkt(wkt_string) => SpatialRef::from_wkt(&wkt_string),
        Srs::Esri(esri_wkt) => SpatialRef::from_esri(&esri_wkt),
        Srs::Epsg(epsg_code) => SpatialRef::from_epsg(epsg_code),
    }
    .unwrap();
    state.with_current_dataset_mut_fallible(|ds, _| {
        ds.dataset
            .dataset
            .set_spatial_ref(&srs)
            .map_err(|err| ErrorDetails::Other(err.to_string()))
    });
}

#[tauri::command]
#[specta::specta]
pub fn run_tool(tool_id: Uuid, parameters: Vec<ToolParameter>, state: AppState) {
    state.with_project_fallible(|project| {
        let tools = project.get_tools();
        let tool = tools
            .iter()
            .find(|tool| tool.get_id() == tool_id)
            .ok_or_else(|| ErrorDetails::Other(format!("Could not get tool with id {tool_id}")))?;
        let output = tool.run(parameters, &mut project.datasets)?;
        let action = tool.get_output_actions();
        for file in &action.load_layers {
            let Some(path) = output.files.get(*file) else {
                return Err(ErrorDetails::Other(
                    "Could not get file from output of tool".to_string(),
                ));
            };
            project
                .datasets
                .open(path, &project.settings)
                .map_err(ErrorDetails::OpenDatasetError)?;
        }
        project.tool_outputs.push(SavedToolOutputAction {
            read: !action.alert_output,
            tool: tool.get_label(),
            output,
            id: Uuid::new_v4(),
        });
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn mark_tool_output_read(id: Uuid, state: AppState) {
    state.with_project_fallible(|project| {
        project
            .tool_outputs
            .iter_mut()
            .find(|output| output.id == id)
            .ok_or_else(|| {
                ErrorDetails::Other("Couldn't find tool output to mark read".to_string())
            })?
            .read = true;
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn add_custom_tool(tool: NewUserDefinedTool, state: AppState) {
    state.with_project(|project| project.tools.push(Box::new(UserDefinedTool::from(tool))));
}

#[tauri::command]
#[specta::specta]
pub fn add_workflow(workflow: NewWorkflow, state: AppState) {}

#[tauri::command]
#[specta::specta]
pub fn run_workflow(workflow: RuntimeInputs, state: AppState) {}

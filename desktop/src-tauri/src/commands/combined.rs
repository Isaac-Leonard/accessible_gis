use std::path::PathBuf;

use itertools::Itertools;
use tauri::AppHandle;
use uuid::Uuid;

use crate::{
    errors::ErrorDetails,
    gdal_if::Srs,
    state::{
        AppState,
        gis::combined::StatefulLayerEnum,
        tools::{NewUserDefinedTool, SavedToolOutputAction, Tool, ToolParameter, UserDefinedTool},
        workflows::{NewWorkflow, Workflow, WorkflowInput},
    },
};

#[tauri::command]
#[specta::specta]
pub fn reproject_layer(srs: Srs, name: &str, state: AppState) {
    state.with_current_layer_mut_fallible(|layer| -> Result<(), ErrorDetails> {
        Ok(match layer {
            StatefulLayerEnum::Vector(layer) => {
                let output = layer
                    .reproject(name, srs)
                    .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
                eprintln!("{:?}", output)
            }
            StatefulLayerEnum::Raster(band) => {
                // TODO: Allow users to specify option for expand_rgba
                let output = band
                    .reproject(name, srs)
                    .map_err(|err| ErrorDetails::IoError(err.to_string()))?;
                eprintln!("{:?}", output)
            }
        })
    });
}

#[tauri::command]
#[specta::specta]
pub fn run_tool(tool_id: Uuid, parameters: Vec<ToolParameter>, state: AppState, app: AppHandle) {
    state.with_project_fallible(|project| {
        let tools = project.get_tools();
        let tool = tools
            .iter()
            .find(|tool| tool.get_id() == tool_id)
            .ok_or_else(|| ErrorDetails::Other(format!("Could not get tool with id {tool_id}")))?;
        let output = tool.run(parameters, &mut project.datasets, &app)?;
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
pub fn add_workflow(workflow: NewWorkflow, state: AppState) {
    state.with_project(|project| project.workflows.push(Workflow::create(workflow)));
}

#[tauri::command]
#[specta::specta]
pub fn run_workflow(id: Uuid, inputs: Vec<WorkflowInput>, state: AppState, app: AppHandle) {
    state.with_project_fallible(|project| {
        project
            .workflows
            .iter()
            .find(|workflow| workflow.id == id)
            .ok_or_else(|| ErrorDetails::Other("Couldn't find project to run".to_string()))?
            .clone()
            .run_workflow(inputs, project, &app)
    });
}

#[tauri::command]
#[specta::specta]
pub fn save_tools_bulk(ids: Vec<Uuid>, file: PathBuf, state: AppState) {
    state.with_project_fallible(|project| {
        let tools_to_save = project
            .get_tools()
            .into_iter()
            .filter_map(|tool| tool.as_user_defined_tool())
            .filter(|tool| ids.contains(&tool.get_id()))
            .collect_vec();
        let json = serde_json::to_string_pretty(&tools_to_save).map_err(|err| {
            ErrorDetails::SerdeError(format!("Failed to serialise tools: {err:?}"))
        })?;
        std::fs::write(file, json)
            .map_err(|err| ErrorDetails::IoError(format!("Failed to save tools: {err:?}")))
    });
}

#[tauri::command]
#[specta::specta]
pub fn save_tool(id: Uuid, file: PathBuf, state: AppState) {
    state.with_project_fallible(|project| {
        let tool_to_save = project
            .get_tools()
            .into_iter()
            .filter_map(|tool| tool.as_user_defined_tool())
            .find(|tool| id == tool.get_id())
            .ok_or_else(|| ErrorDetails::Other("Could not find tool to save".to_string()))?;
        let json = serde_json::to_string_pretty(&tool_to_save).map_err(|err| {
            ErrorDetails::SerdeError(format!("Failed to serialise tools: {err:?}"))
        })?;
        std::fs::write(file, json)
            .map_err(|err| ErrorDetails::IoError(format!("Failed to save tools: {err:?}")))
    });
}

#[tauri::command]
#[specta::specta]
pub fn load_tool(file: PathBuf, state: AppState) {
    state.with_project_fallible(|project| {
        let content = std::fs::read(file).map_err(|err| {
            ErrorDetails::IoError(format!("Could not read file to load tool: {err:?}"))
        })?;
        let tool = serde_json::from_slice::<UserDefinedTool>(&content).map_err(|err| {
            ErrorDetails::SerdeError(format!("Could not deserialise saved tool: {err:?}"))
        })?;
        project.tools.push(Box::new(tool));
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn load_tools_bulk(file: PathBuf, state: AppState) {
    state.with_project_fallible(|project| {
        let content = std::fs::read(file).map_err(|err| {
            ErrorDetails::IoError(format!("Could not read file to load tools: {err:?}"))
        })?;
        let tools = serde_json::from_slice::<Vec<UserDefinedTool>>(&content).map_err(|err| {
            ErrorDetails::SerdeError(format!("Could not deserialise saved tools: {err:?}"))
        })?;
        project.tools.extend(
            tools
                .into_iter()
                .map(|tool| Box::new(tool) as Box<dyn Tool>),
        );
        Ok(())
    });
}

#[tauri::command]
#[specta::specta]
pub fn remove_dataset(state: AppState) {
    state.with_project(|project| project.datasets.remove_dataset());
}

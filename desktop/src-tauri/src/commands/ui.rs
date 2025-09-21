use itertools::Itertools;
use uuid::Uuid;

use crate::{
    gdal_if::list_drivers,
    state::{AppState, Screen, workflows::Workflow},
    ui::{
        NewDatasetScreenData, ProjectScreen, ToolsScreenInfo, TouchDeviceState, UiScreen, UiState,
        WorkflowsScreenInfo,
    },
};

#[tauri::command]
#[specta::specta]
pub fn set_screen(screen: Screen, state: AppState) {
    let mut guard = state.data.lock().unwrap();
    guard.screen = screen;
}

#[tauri::command]
#[specta::specta]
pub fn get_app_info(state: AppState) -> UiState {
    state.with_lock(|state| UiState {
        screen: match state.screen {
            Screen::Main => UiScreen::Project(
                state
                    .get_project_screen_info()
                    .map(ProjectScreen::Project)
                    .unwrap_or_default(),
            ),
            Screen::NewDataset => UiScreen::NewDataset(NewDatasetScreenData {
                // Kind of hacky but this function cannot return an error
                drivers: list_drivers()
                    .map_err(|err| state.errors.push(err.into()))
                    .unwrap_or_default(),
            }),
            Screen::Settings => UiScreen::Settings(state.settings().clone()),
            Screen::Tools => UiScreen::Tools(
                state
                    .with_project_fallible(|project| {
                        Ok(ToolsScreenInfo {
                            tools: project.tools.iter().map(|tool| tool.for_ui()).collect(),
                            layers: project
                                .datasets
                                .get_all_layers()?
                                .into_iter()
                                .map_into()
                                .collect(),
                        })
                    })
                    .unwrap_or_default(),
            ),
            Screen::Workflows => UiScreen::Workflows(
                state
                    .with_project_fallible(|project| {
                        Ok(WorkflowsScreenInfo {
                            tools: project.tools.iter().map(|tool| tool.for_ui()).collect(),
                            workflows: project.workflows.iter().map(Workflow::for_ui).collect(),
                            layers: project
                                .datasets
                                .get_all_layers()?
                                .into_iter()
                                .map_into()
                                .collect(),
                        })
                    })
                    .unwrap_or_default(),
            ),
            Screen::TouchDevice => UiScreen::TouchDevice(TouchDeviceState {}),
            Screen::Errors => UiScreen::Errors,
        },
        errors: state.errors.to_vec(),
        tool_outputs: state
            .with_project(|p| p.tool_outputs.clone())
            .unwrap_or_default(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn mark_error_read(id: Uuid, state: AppState) {
    state.with_lock(|state| state.errors.get(id).unwrap().read = true)
}

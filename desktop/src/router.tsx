import { MainScreen } from "./main-screen";
import { NewDatasetScreen } from "./new-dataset-screen";
import { state } from "./api";
import { SettingsScreen } from "./settings-screen";
import { TouchDeviceScreen } from "./touch-device-screen";
import { ErrorsScreen } from "./errors-screen";
import { ToolsScreen } from "./tools-screen";
import { WorkflowScreen } from "./workflow-screen";

export const Router = () => {
  switch (state.value.screen.name) {
    case "Project":
      return <MainScreen state={state.value.screen} />;
    case "NewDataset":
      return <NewDatasetScreen drivers={state.value.screen.drivers} />;
    case "Settings":
      return <SettingsScreen settings={state.value.screen} />;
    case "TouchDevice":
      return <TouchDeviceScreen state={state.value.screen} />;
    case "Tools":
      return <ToolsScreen {...state.value.screen} />;
    case "Workflows":
      return <WorkflowScreen info={state.value.screen} />;
    case "Errors":
      return <ErrorsScreen errors={state.value.errors} />;
  }
};

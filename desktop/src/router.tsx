import { ThiessenPolygons } from "./thiessen-polygons-screen";
import { MainScreen } from "./main-screen";
import { NewDatasetScreen } from "./new-dataset-screen";
import { state } from "./api";
import { SettingsScreen } from "./settings-screen";
import { TouchDeviceScreen } from "./touch-device-screen";
import { ErrorsScreen } from "./errors-screen";

export const Router = () => {
  switch (state.value.screen.name) {
    case "Project":
      return <MainScreen state={state.value.screen} />;
    case "ThiessenPolygons":
      return <ThiessenPolygons />;
    case "NewDataset":
      return <NewDatasetScreen drivers={state.value.screen.drivers} />;
    case "Settings":
      return <SettingsScreen settings={state.value.screen} />;
    case "TouchDevice":
      return <TouchDeviceScreen use_labels={state.value.screen.use_labels} />;
    case "Errors":
      return <ErrorsScreen errors={state.value.errors} />;
  }
};

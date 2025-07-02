import { ThiessenPolygons } from "./thiessen-polygons-screen";
import { MainScreen } from "./main-screen";
import { NewDatasetScreen } from "./new-dataset-screen";
import { state } from "./api";
import { SettingsScreen } from "./settings-screen";
import { LayerScreenContext } from "./context";
import { TouchDeviceScreen } from "./touch-device-screen";
import { ErrorsScreen } from "./errors-screen";

export const Router = () => {
  switch (state.value.screen.name) {
    case "Layers":
      return (
        <LayerScreenContext.Provider value={state.value.screen}>
          <MainScreen state={state.value.screen} />
        </LayerScreenContext.Provider>
      );
    case "ThiessenPolygons":
      return <ThiessenPolygons />;
    case "NewDataset":
      return <NewDatasetScreen drivers={state.value.screen.drivers} />;
    case "Settings":
      return <SettingsScreen settings={state.value.screen} />;
    case "TouchDevice":
      return <TouchDeviceScreen />;
    case "Errors":
      <ErrorsScreen errors={state.value.errors} />;
  }
};

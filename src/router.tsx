import { ThiessenPolygons } from "./thiessen-polygons-screen";
import { MainScreen } from "./main-screen";
import { NewDatasetScreen } from "./new-dataset-screen";
import { state } from "./api";
import { SettingsScreen } from "./settings-screen";
import { LayerScreenContext } from "./context";

export const Router = () => {
  switch (state.value.name) {
    case "Layers":
      return (
        <LayerScreenContext.Provider value={state.value}>
          {" "}
          <MainScreen state={state.value} />
        </LayerScreenContext.Provider>
      );
    case "ThiessenPolygons":
      return <ThiessenPolygons />;
    case "NewDataset":
      return <NewDatasetScreen drivers={state.value.drivers} />;
    case "Settings":
      return <SettingsScreen settings={state.value} />;
  }
};

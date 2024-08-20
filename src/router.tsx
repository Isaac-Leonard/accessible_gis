import { ThiessenPolygons } from "./thiessen-polygons-screen";
import { MainScreen } from "./main-screen";
import { NewDatasetScreen } from "./new-dataset-screen";
import { state } from "./api";
import { SettingsScreen } from "./settings-screen";

export const Router = () => {
  switch (state.value.name) {
    case "Layers":
      return <MainScreen state={state.value} />;
    case "ThiessenPolygons":
      return <ThiessenPolygons />;
    case "NewDataset":
      return <NewDatasetScreen drivers={state.value.drivers} />;
    case "Settings":
      return <SettingsScreen settings={state.value} />;
  }
};

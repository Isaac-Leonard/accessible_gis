import { ThiessenPolygons } from "./thiessen-polygons-screen";
import { MainScreen } from "./main-screen";
import { NewDatasetScreen } from "./new-dataset-screen";
import { state } from "./api";

export const Router = () => {
  switch (state.value.screen.name) {
    case "Main":
      return <MainScreen />;
    case "ThiessenPolygons":
      return <ThiessenPolygons />;
    case "NewDataset":
      return <NewDatasetScreen drivers={state.value.screen.drivers} />;
  }
};

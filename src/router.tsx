import { useContext } from "react";
import { context } from "./context";
import { ThiessenPolygons } from "./thiessen-polygons-screen";
import { MainScreen } from "./main-screen";

export const Router = () => {
  const { screen } = useContext(context);
  switch (screen) {
    case "main":
      return <MainScreen />;
    case "thiessen_polygons":
      return <ThiessenPolygons />;
  }
};

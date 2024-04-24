import { createContext } from "react";

export type Screen = "main" | "thiessen_polygons";
export type State = { screen: Screen; setScreen: (screen: Screen) => void };

export const context = createContext<State>({
  screen: "main",
  setScreen() {
    throw "Context provider not set";
  },
});
export const Provider = context.Provider;

import { useState } from "react";
import "./App.css";
import { Provider, Screen } from "./context";
import { Router } from "./router";

function App() {
  const [screen, setScreen] = useState<Screen>("main");
  return (
    <Provider value={{ screen, setScreen }}>
      <button onClick={() => setScreen("main")}>Main</button>
      <button onClick={() => setScreen("thiessen_polygons")}>
        Thiessen Polygons
      </button>
      <Router />
    </Provider>
  );
}

export default App;

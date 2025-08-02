import "./App.css";
import { Router } from "./router";
import { client, state } from "./api";
import { load, newProject } from "./files";
import { ErrorsPopup } from "./errors-screen";
import { ToolOutputsPopup } from "./tools-screen";

const globalKeyHandler = (e: KeyboardEvent) => {
  if (e.metaKey) {
    switch (e.key) {
      case "o":
        e.preventDefault();
        load();
        break;
      case "n":
        e.preventDefault();
        newProject();
        break;
      case "s":
        e.preventDefault();
        client.saveProject();
        break;
    }
  }
};

function App() {
  return (
    <div onKeyDown={globalKeyHandler}>
      <ErrorsPopup />
      <ToolOutputsPopup />

      <button onClick={() => client.setScreen("Main")}>Main</button>
      <button onClick={() => client.setScreen("NewDataset")}>
        New dataset{" "}
      </button>
      <button onClick={() => client.openSettings()}>Settings</button>
      <button onClick={() => client.setScreen("Tools")}>Tools</button>
      <button onClick={() => client.setScreen("TouchDevice")}>
        Touch Device
      </button>
      <button onClick={() => client.setScreen("Errors")}>Errors</button>
      <Router />
    </div>
  );
}

export default App;

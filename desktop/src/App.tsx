import "./App.css";
import { Router } from "./router";
import { client } from "./api";
import { load, openFile } from "./files";
import { ErrorsPopup } from "./errors-screen";
import { ToolOutputsPopup } from "./tools-screen";

const globalKeyHandler = async (e: KeyboardEvent) => {
  if (e.metaKey) {
    switch (e.key) {
      case "o":
        e.preventDefault();
        load();
        break;
      case "n":
        e.preventDefault();
        let file = await openFile("Project location");
        if (file) {
          client.createProject(file);
        }
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
        New dataset
      </button>
      <button onClick={() => client.setScreen("Settings")}>Settings</button>
      <button onClick={() => client.setScreen("Tools")}>Tools</button>
      <button onClick={() => client.setScreen("Workflows")}>Workflows</button>
      <button onClick={() => client.setScreen("TouchDevice")}>
        Touch Device
      </button>
      <button onClick={() => client.setScreen("Errors")}>Errors</button>
      <Router />
    </div>
  );
}

export default App;

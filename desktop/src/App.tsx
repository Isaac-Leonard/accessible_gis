import "./App.css";
import { Router } from "./router";
import { client } from "./api";
import { load } from "./files";

const globalKeyHandler = (e: KeyboardEvent) => {
  if (e.metaKey) {
    switch (e.key) {
      case "o":
        e.preventDefault();
        load();
        break;
    }
  }
};

function App() {
  return (
    <div onKeyDown={globalKeyHandler}>
      <button onClick={() => client.setScreen("Main")}>Main</button>
      <button onClick={() => client.setScreen("NewDataset")}>
        New dataset
      </button>
      <button onClick={() => client.openSettings()}>Settings</button>
      <button onClick={() => client.setScreen("TouchDevice")}>
        Touch Device
      </button>
      <Router />
    </div>
  );
}

export default App;

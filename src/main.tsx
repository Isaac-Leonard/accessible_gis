import React, { Suspense } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Suspense fallback=" loading">
      <App />
    </Suspense>
  </React.StrictMode>
);

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { attachConsole } from "tauri-plugin-log-api";

//REVIEW - maybe detach browser console here?
attachConsole();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

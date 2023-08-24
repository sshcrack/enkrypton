import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.scss";
import { attachConsole } from "tauri-plugin-log-api";
import {
  FluentProvider,
  webDarkTheme
} from "@fluentui/react-components";

//REVIEW - maybe detach browser console here?
attachConsole();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <FluentProvider theme={webDarkTheme} className='full-size'>
      <App />
    </FluentProvider>
  </React.StrictMode>,
);

import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import "./styles.scss";
import { attachConsole } from "tauri-plugin-log-api";
import {
  FluentProvider,
  webDarkTheme
} from "@fluentui/react-components";
import { UnlistenFn } from '@tauri-apps/api/event';

const ConsoleListener = ({ children }: React.PropsWithChildren<{}>) => {
  useEffect(() => {
    let unlisten: UnlistenFn;
    attachConsole().then(e => unlisten = e);

    return () => {
      if (!unlisten)
        return;

      unlisten()
    }
  }, [])

  return <>{children}</>
}

export function renderPage(Page: () => JSX.Element) {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ConsoleListener>
        <FluentProvider theme={webDarkTheme} className='full-size'>
          <Page />
        </FluentProvider>
      </ConsoleListener>
    </React.StrictMode>,
  );
}
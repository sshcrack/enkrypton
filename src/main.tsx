import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import "./styles.scss";
import { attachConsole } from "tauri-plugin-log-api";
import { UnlistenFn } from '@tauri-apps/api/event';
import { ChakraProvider, useColorMode } from '@chakra-ui/react';
import { Dict } from '@chakra-ui/utils';

const ConsoleListener = ({ children }: React.PropsWithChildren<{}>) => {
  const { colorMode, toggleColorMode } = useColorMode()

  if (colorMode === "light")
    toggleColorMode()

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

const theme: Dict = {
}

export function renderPage(Page: () => JSX.Element) {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ChakraProvider theme={theme}>
        <ConsoleListener>
          <Page />
        </ConsoleListener>
      </ChakraProvider>
    </React.StrictMode>,
  );
}
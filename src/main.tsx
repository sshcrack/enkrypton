import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import "./styles.scss";
import { attachConsole } from "tauri-plugin-log-api";
import { UnlistenFn } from '@tauri-apps/api/event';
import { ChakraProvider, extendTheme, useColorMode } from '@chakra-ui/react';
import { Dict } from '@chakra-ui/utils';

const ConsoleListener = ({ children }: React.PropsWithChildren<{}>) => {
  const { colorMode, toggleColorMode } = useColorMode()

  useEffect(() => {
    if (colorMode === "light") {
      console.log("Color mode is light, toggling")
      toggleColorMode()
    }
  }, [colorMode])

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

const theme: Dict = extendTheme({
})

export function renderPage(Page: () => React.JSX.Element) {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ChakraProvider theme={theme}>
        <ConsoleListener>
          <Page />
        </ConsoleListener>
      </ChakraProvider>
    </React.StrictMode>
  );
}
import { invoke } from "@tauri-apps/api/tauri";
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useEffect } from "react"
import "./App.css";

function App() {
  useEffect(() => {
    let unlisten: UnlistenFn = () => {};
    listen('tor_start', (event) => {
    }).then(e => unlisten = e);

    return () => unlisten()
  }, [])

  const startTor = async () => {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("tor_start")
  }

  return (
    <div className="container">
      <button onClick={() => startTor()}>Start Tor</button>
    </div>
  );
}

export default App;

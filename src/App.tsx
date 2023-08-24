import { invoke } from "@tauri-apps/api/tauri";
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import "./App.scss";
import { Button, ProgressBar, Text } from '@fluentui/react-components';
import { StartTorPayload } from './payloads/StartTorPayload';

function App() {
  const [ percentage, setCurrPercentage] = useState(0);
  const [ status, setStatus] = useState("Initializing...");

  useEffect(() => {
    let unlisten: UnlistenFn = () => {};
    listen('tor_start', (event) => {
      const payload = event.payload as StartTorPayload;

      setCurrPercentage(payload.progress)
      setStatus(payload.message)
    }).then(e => unlisten = e);

    return () => unlisten()
  }, [])

  const startTor = async () => {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("tor_start")
  }

  return (
    <div className="full-size flex-center flex flex-column">
      <Button onClick={() => startTor()}>Start Tor</Button>
      <div className='tor-status-wrapper flex flex-column flex-center'>
        <Text>{status}</Text>
        <ProgressBar value={percentage} />
      </div>
    </div>
  );
}

export default App;

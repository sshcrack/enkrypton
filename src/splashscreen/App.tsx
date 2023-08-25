import { listen, UnlistenFn, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import "./App.scss";
import { ProgressBar, Text } from '@fluentui/react-components';
import { StartTorPayload } from './payloads/StartTorPayload';
//import { info, error } from "tauri-plugin-log-api";

function App() {
  const [ percentage, setCurrPercentage] = useState(0);
  const [ status, setStatus] = useState("Initializing...");

  useEffect(() => {
    let unlisten: UnlistenFn = () => {};
    listen('tor_start', (event) => {
      const payload = event.payload as StartTorPayload;

      setCurrPercentage(payload.progress)
      setStatus(payload.message)
    }).then(e => {
      unlisten = e
      emit("splashscreen_ready")
    })

    return () => unlisten()
  }, [])

  return (
    <div className="full-size flex-center flex flex-column">
      <div className='tor-status-wrapper flex flex-column flex-center'>
        <Text>{status}</Text>
        <ProgressBar value={percentage} />
      </div>
    </div>
  );
}

export default App;

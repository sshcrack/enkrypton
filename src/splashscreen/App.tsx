import { listen, UnlistenFn, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import "./App.scss";
import { ProgressBar, Text } from '@fluentui/react-components';
import { StartTorPayload } from './payloads/StartTorPayload';
//import { info, error } from "tauri-plugin-log-api";

function App() {
  const [ percentage, setCurrPercentage] = useState(0);
  const [ status, setStatus] = useState("Initializing...");
  const [ error, setError ] = useState<string>(undefined)

  useEffect(() => {
    let unlisten_start: UnlistenFn = () => {};
    let unlisten_error: UnlistenFn = () => {};

    listen('tor_start', (event) => {
      const payload = event.payload as StartTorPayload;

      setCurrPercentage(payload.progress)
      setStatus(payload.message)
    }).then(e => {
      listen("tor_start_error", (event) => {

      })
      unlisten_start = e
      emit("splashscreen_ready")
    })

    return () => {
      unlisten_start && unlisten_start();
      unlisten_error && unlisten_error()
    }
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

import { listen, UnlistenFn, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import "./App.scss";
import { StartTorErrorPayload, StartTorPayload } from './payloads/StartTorPayload';
import ErrorScreen from './components/error';
import LoadingScreen from './components/loading';
import { Text } from '@chakra-ui/react';
//import { info, error } from "tauri-plugin-log-api";

function App() {
  const [percentage, setCurrPercentage] = useState(0);
  const [status, setStatus] = useState("Initializing...");
  const [error, setError] = useState<StartTorErrorPayload | null>(null)
  console.log("app page")

  useEffect(() => {
    let unlisten_start: UnlistenFn = () => { };
    let unlisten_error: UnlistenFn = () => { };

    listen('tor_start', (event) => {
      const payload = event.payload as StartTorPayload;

      setCurrPercentage(payload.progress)
      setStatus(payload.message)
    }).then(e => {
      unlisten_start = e
      //emit("splashscreen_ready")

      return listen("tor_start_error", (event) => {
        let payload = event.payload as StartTorErrorPayload;

        setError(payload)
      })
    }).then(e => {
      unlisten_error = e
    })

    return () => {
      unlisten_start && unlisten_start();
      unlisten_error && unlisten_error()
    }
  }, [])


  return error ? <ErrorScreen error={error} /> : <LoadingScreen progress={percentage} status={status} />
}

export default App;

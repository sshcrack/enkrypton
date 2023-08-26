import { listen, UnlistenFn, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import "./App.scss";
import { Button, ProgressBar, Text, Title3 } from '@fluentui/react-components';
import { StartTorErrorPayload, StartTorPayload } from './payloads/StartTorPayload';
import { invoke } from '@tauri-apps/api/tauri';
import { relaunch } from '@tauri-apps/api/process';
//import { info, error } from "tauri-plugin-log-api";

function App() {
  const [percentage, setCurrPercentage] = useState(0);
  const [status, setStatus] = useState("Initializing...");
  const [error, setError] = useState<StartTorErrorPayload | null>(null)

  useEffect(() => {
    let unlisten_start: UnlistenFn = () => { };
    let unlisten_error: UnlistenFn = () => { };

    listen('tor_start', (event) => {
      const payload = event.payload as StartTorPayload;

      setCurrPercentage(payload.progress)
      setStatus(payload.message)
    }).then(e => {
      unlisten_start = e
      emit("splashscreen_ready")

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

  const logs = error?.logs?.concat([])?.reverse()
    // filtering date out
    .map(e => {
      let index = e.indexOf("[");
      if (index === -1)
        index = 0

      return e.substring(index)
    });

  return (
    <div className={`full-size flex-center flex flex-column ${error && "error"}`}>
      <div className='tor-status-wrapper flex flex-column flex-center'>
        {!error && <Text>{status}</Text>}
        {!error && <ProgressBar value={percentage} />}

        {error && logs && <>
          <Title3>Could not start tor</Title3>
          <Text>Exit code: {error.error_code}</Text>
          <br />
          <Text className="log-style">{logs.join("\n")}</Text>
        </>}
      </div>
      {error && <Button
        style={{ borderRadius: "0px" }}
        onClick={() => relaunch()}
      >Retry</Button>}
    </div>
  );
}

export default App;

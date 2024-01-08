import { useEffect, useState } from "react"
import "./App.scss";
import ErrorScreen from './components/error';
import LoadingScreen from './components/loading';
import { TorStartupErrorPayload } from '../bindings/rs/TorStartupErrorPayload';
import { listenSync } from '../bindings/tauri_prom_wrapper';
import tor from '../bindings/tor';
//import { info, error } from "tauri-plugin-log-api";

function TorStart() {
    const [percentage, setCurrPercentage] = useState(0);
    const [status, setStatus] = useState("Initializing...");
    const [error, setError] = useState<TorStartupErrorPayload | null>(null)
    console.log("app page")

    useEffect(() => {
        const unlisten_start = listenSync("tor_start", ({ payload }) => {
            setCurrPercentage(payload.progress)
            setStatus(payload.message)
        });

        let unlisten_error = listenSync("tor_start_error", ({ payload }) => setError(payload));
        tor.send_ready().catch(console.error);

        return () => {
            unlisten_start && unlisten_start();
            unlisten_error && unlisten_error()
        }
    }, [])


    return error ? <ErrorScreen error={error} /> : <LoadingScreen progress={percentage} status={status} />
}

export default TorStart;

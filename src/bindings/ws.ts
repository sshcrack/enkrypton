import { invoke } from "@tauri-apps/api"

const ws = {
    connect: (onionAddr: string) => invoke("ws_connect", { onionAddr }),
    send: (onionAddr: string, msg: string) => invoke("ws_send", { onionAddr, msg })
}

export default ws;
import { emit } from '@tauri-apps/api/event'
import { invoke } from "@tauri-apps/api"

const tor = {
    send_ready: () => emit("splashscreen_ready"),
    get_hostname: () => invoke("tor_hostname") as Promise<string>
}

export default tor;
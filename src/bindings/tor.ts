import { emit } from '@tauri-apps/api/event'
import { invoke } from "@tauri-apps/api"

const tor = {
    send_ready: () => emit("splashscreen_ready"),
    is_splashscreen_closed: () => invoke("splashscreen_closed") as Promise<boolean>,
    get_hostname: () => invoke("tor_hostname") as Promise<string>
}

export default tor;
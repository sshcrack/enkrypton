import { emit } from '@tauri-apps/api/event'
import { invoke } from "@tauri-apps/api/core"

/**
 * Contains all bindings for the backend tor module
 */
const tor = {
    /**
     * 
     * @returns a promise which is resolved once tor is started.
     */
    send_ready: () => emit("splashscreen_ready"),
    /**
     * Checks if the splashscreen is closed
     * @returns a promise which returns whether the splashscreen is closed or not
     */
    is_splashscreen_closed: () => invoke("splashscreen_closed") as Promise<boolean>,
    /**
     * Gets teh hostname of the tor instance
     * @returns the tor hostname
     */
    get_hostname: () => invoke("tor_hostname") as Promise<string>
}

export default tor;
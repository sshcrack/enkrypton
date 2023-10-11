import { invoke } from "@tauri-apps/api"

const tor = {
    exists: () => invoke("storage_exists") as Promise<boolean>,
    unlockOrCreate: (pass: string) => invoke("storage_unlock_or_create", { pass }) as Promise<void>,
    is_unlocked: () => invoke("storage_is_unlocked") as Promise<boolean>,
}

export default tor;
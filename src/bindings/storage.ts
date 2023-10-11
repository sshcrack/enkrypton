import { invoke } from "@tauri-apps/api"
import { StorageData } from './rs/StorageData';

const storage = {
    exists: () => invoke("storage_exists") as Promise<boolean>,
    unlockOrCreate: (pass: string) => invoke("storage_unlock_or_create", { pass }) as Promise<void>,
    is_unlocked: () => invoke("storage_is_unlocked") as Promise<boolean>,
    set: (data: StorageData) => {
        const raw = JSON.stringify(data)
        return invoke("storage_set", { dataRaw: raw }) as Promise<void>
    },
    get: () => invoke("storage_get") as Promise<StorageData>,
    save: () => invoke("storage_save") as Promise<void>
}

export default storage;
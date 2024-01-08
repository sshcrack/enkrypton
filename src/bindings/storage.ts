import { invoke } from "@tauri-apps/api"
import { StorageData } from './rs/StorageData';
import { listen, Event } from "@tauri-apps/api/event"

type Func = () => unknown;
const listeners: Func[] = [];
const storage = {
    exists: () => invoke("storage_exists") as Promise<boolean>,
    unlockOrCreate: (pass: string) => invoke("storage_unlock_or_create", { pass }) as Promise<void>,
    is_unlocked: () => invoke("storage_is_unlocked") as Promise<boolean>,
    set: (data: StorageData) => {
        const raw = JSON.stringify(data)
        return invoke("storage_set", { dataRaw: raw }) as Promise<void>
    },
    get: () => invoke("storage_get") as Promise<StorageData>,
    save: () => invoke("storage_save") as Promise<void>,
    onStorageDirty: (callback: () => unknown) => {
        listeners.push(callback)

        return () => {
            const index = listeners.indexOf(callback)
            if (index === -1)
                return console.error("Could not remove manual listener")

            listeners.splice(index, 1)
        }
    }
}

listen("storage_changed", ({ }: Event<{}>) => {
    listeners.map(l => l())
    console.log("Storage changed")
}).catch(console.error)

export default storage;
import { invoke } from "@tauri-apps/api/core"
import { StorageData } from './rs/StorageData';
import { listen, Event } from "@tauri-apps/api/event"

type Func = () => unknown;
const listeners: Func[] = [];
const storage = {
    /**
     * Checks if the storage exists
     * @returns Wheter the storage exists or not
     */
    exists: () => invoke("storage_exists") as Promise<boolean>,
    /**
     * Unlocks or creates the storage with the given password
     * @param pass the password to use for unlocking or creating the storage
     * @returns a promise which is resolved once unlocked
     */
    unlockOrCreate: (pass: string) => invoke("storage_unlock_or_create", { pass }) as Promise<void>,
    /**
     * Checks if the storage is unlocked.
     * @returns a boolean indicating if the storage is unlocked.
     */
    is_unlocked: () => invoke("storage_is_unlocked") as Promise<boolean>,
    /**
     * Sends the data to backend and schedules it to save.
     * @param data The data to schedule for saving.
     * @returns a promise which is resolved once the data is saved.
     */
    set: (data: StorageData) => {
        const raw = JSON.stringify(data)
        return invoke("storage_set", { dataRaw: raw }) as Promise<void>
    },
    /**
     * Gets the storage data.
     * @returns a promise with the data from storage.
     */
    get: () => invoke("storage_get") as Promise<StorageData>,
    /**
     * Saves the storage data to disk.
     */
    save: () => invoke("storage_save") as Promise<void>,
    /**
     * Fires the callback when the storage was modified.
     * @param callback The callback to fire when the storage was modified.
     * @returns a function to remove the listener.
     */
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

// Notifies listeners
listen("storage_changed", ({ }: Event<{}>) => {
    listeners.map(l => l())
    console.log("Storage changed")
}).catch(console.error)

export default storage;
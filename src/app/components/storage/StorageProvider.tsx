import React, { useEffect, useState } from 'react';
import { StorageData } from '../../../bindings/rs/StorageData';
import storage from '../../../bindings/storage';

export type StorageContextState = {
    data: StorageData | null;
}

export const StorageContext = React.createContext<StorageContextState>({} as StorageContextState);

/**
 * Use in Main App Only
 */
export function StorageProvider({ children }: React.PropsWithChildren<{}>) {
    const [data, setData] = useState<StorageData | null>(null);
    const [locked, setLocked] = useState(true)


    // Just initial set of data
    useEffect(() => {
        if (locked)
            return console.debug("Locked, not listening to event")

        return storage.onStorageDirty(() => {
            setLocked(true)
            storage.get()
                .then(e => {
                    setData(e)
                    setLocked(false)
                })
        })
    }, [locked])

    useEffect(() => {
        storage.get()
            .then(e => {
                setData(e)
                setLocked(false)
            })
    }, [])


    return <StorageContext.Provider value={{
        data
    }}>
        {children}
    </StorageContext.Provider>
}
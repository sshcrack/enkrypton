import React, { useEffect, useState } from 'react';
import { useDebounce } from 'usehooks-ts';
import { StorageData } from '../../../bindings/rs/StorageData';
import storage from '../../../bindings/storage';
import { ReactSetState } from '../../../tools/react';

export type StorageContextState = {
    data: StorageData | null;
    setData: ReactSetState<StorageData | null>;
}

export const StorageContext = React.createContext<StorageContextState>({} as StorageContextState);

/**
 * Use in Main App Only
 */
export function StorageProvider({ children }: React.PropsWithChildren<{}>) {
    const [data, setData] = useState<StorageData | null>(null);
    const [waitingForUpdate, setWaitingForUpdate] = useState(false);

    const [update, setUpdate] = useState(0)
    const [locked, setLocked] = useState(true)

    const debouncedData = useDebounce(data, 350)

    useEffect(() => {
        if (!debouncedData || locked || !waitingForUpdate)
            return

        setLocked(true)
        storage.set(debouncedData)
            .then(() => {
                console.log("Saved chat data")
            })
            .catch(e => console.error(e))
            .finally(() => {
                setLocked(false)
                setWaitingForUpdate(false)
            })
    }, [debouncedData, locked, waitingForUpdate])

    // Just initial set of data
    useEffect(() => {
        storage.get()
            .then(e => {
                setData(e)
                setLocked(false)
            })
            .catch(e => {
                console.error(e)
                setTimeout(() => {
                    setUpdate(update + 1)
                }, 100)
            })
    }, [update])


    return <StorageContext.Provider value={{
        data,
        setData: d => {
            setWaitingForUpdate(true)
            setData(d)
        }
    }}>
        {children}
    </StorageContext.Provider>
}
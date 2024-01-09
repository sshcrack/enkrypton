import React, { useEffect, useState } from 'react';
import { StorageData } from '../../../bindings/rs/StorageData';
import storage from '../../../bindings/storage';
import { Flex, Spinner, Text } from '@chakra-ui/react';

export type StorageContextState = {
    /**
     * The storage data that is currently loaded from the backend.
     */
    data: StorageData | null;
}

/**
 * The context for the storage data.
 */
export const StorageContext = React.createContext<StorageContextState>({} as StorageContextState);

/**
 * Provides the storage data to the children.
 * Use in Main App Only!
 */
export function StorageProvider({ children }: React.PropsWithChildren<{}>) {
    const [unlocked, setUnlocked] = useState(false);
    const [recheck, setRecheck] = useState(0)

    // The data itself
    const [data, setData] = useState<StorageData | null>(null);
    const [locked, setLocked] = useState(true)

    useEffect(() => {
        // check if the stoarge is unlocked and update the data
        storage.is_unlocked()
            .then(e => setUnlocked(e))
            .catch(e => {
                setTimeout(() => setRecheck(recheck + 1), 100)
                return console.error("Failed to check if unlocked, retrying", e)
            });
    }, [recheck])


    // Just initial set of data
    useEffect(() => {
        if (!unlocked)
            return

        if (locked)
            return console.debug("Locked, not listening to event")

        // Fire when the storage is dirty (so was modified and is not the same as the local cache anymore)
        return storage.onStorageDirty(() => {
            // Lock the storage while fetching
            setLocked(true)
            storage.get()
                .then(e => {
                    setData(e)
                    setLocked(false)
                })
        })
    }, [locked, unlocked])

    // And update the data when unlocked
    useEffect(() => {
        if (!unlocked)
            return

        storage.get()
            .then(e => {
                setData(e)
                setLocked(false)
            })
    }, [unlocked])

    // Waiting for backend to decrypt data
    if (!unlocked)
        return <Flex w='100%' h='100%' justifyContent='center' alignItems='center'>
            <Flex gap='5'>
                <Spinner />
                <Text>Unlocking...</Text>
            </Flex>
        </Flex>



    // Providing data to backend
    return <StorageContext.Provider value={{
        data
    }}>
        {children}
    </StorageContext.Provider>
}
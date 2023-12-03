import React, { useEffect, useState } from 'react';
import { StorageData } from '../../../bindings/rs/StorageData';
import storage from '../../../bindings/storage';
import { Flex, Spinner, Text } from '@chakra-ui/react';

export type StorageContextState = {
    data: StorageData | null;
}

export const StorageContext = React.createContext<StorageContextState>({} as StorageContextState);

/**
 * Use in Main App Only
 */
export function StorageProvider({ children }: React.PropsWithChildren<{}>) {
    const [unlocked, setUnlocked] = useState(false);
    const [recheck, setRecheck] = useState(0)

    const [data, setData] = useState<StorageData | null>(null);
    const [locked, setLocked] = useState(true)

    useEffect(() => {
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

        return storage.onStorageDirty(() => {
            setLocked(true)
            storage.get()
                .then(e => {
                    setData(e)
                    setLocked(false)
                })
        })
    }, [locked, unlocked])

    useEffect(() => {
        if (!unlocked)
            return
        storage.get()
            .then(e => {
                setData(e)
                setLocked(false)
            })
    }, [unlocked])

    if (!unlocked)
        return <Flex w='100%' h='100%' justifyContent='center' alignItems='center'>
            <Flex gap='5'>
                <Spinner />
                <Text>Unlocking...</Text>
            </Flex>
        </Flex>



    return <StorageContext.Provider value={{
        data
    }}>
        {children}
    </StorageContext.Provider>
}
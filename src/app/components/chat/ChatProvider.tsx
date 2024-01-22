import { Text } from '@chakra-ui/react'
import React, { useContext, useEffect, useState } from 'react'
import MessagingClient from '../../../bindings/ws/client'
import { GeneralUser } from '../../../bindings/ws/client/types'
import { StorageContext } from '../storage/StorageProvider'


export type ChatContextState = {
    /**
     * The client to use for messaging (null if no connection has been made yet).
     */
    client: MessagingClient | null,
    /**
     * Used to force a rerender when a message is received.
     */
    msgUpdate: number
}

export const ChatContext = React.createContext<ChatContextState>({} as ChatContextState)

export default function ChatProvider({ children, user }: React.PropsWithChildren<{ user: GeneralUser }>) {
    // Reading the storage data from cache / backend
    const { data } = useContext(StorageContext)

    // Again, this is the undlying client for messaging
    const [client, setClient] = useState<MessagingClient | null>(null)
    const [msgUpdate, setUpdate] = useState(0)

    const { clients } = window
    useEffect(() => {
        if (!clients)
            return

        // Getting the client (or its default) from all clients currently connected
        const c = clients.get(user.onionHostname);
        setClient(c)

        const listener = () => {
            setUpdate(Math.random())
        }

        // Adding the listener for message updates
        c.addListener("on_update", listener)

        // Has to be removed as well when the component unmounts
        return () => {
            c.removeListener("on_update", listener)
        }
    }, [user])

    if (data === null)
        return <Text>Loading storage data...</Text>

    // Exposing the ChatContext to the children
    return <ChatContext.Provider value={{
        client,
        msgUpdate
    }}>
        {children}
    </ChatContext.Provider>
}
import { Text } from '@chakra-ui/react'
import React, { useContext, useEffect, useState } from 'react'
import MessagingClient from '../../../bindings/ws/client'
import { GeneralUser } from '../../../bindings/ws/client/types'
import { StorageContext } from '../storage/StorageProvider'


export type ChatContextState = {
    client: MessagingClient | null,
    msgUpdate: number
}

export const ChatContext = React.createContext<ChatContextState>({} as ChatContextState)

export default function ChatProvider({ children, user }: React.PropsWithChildren<{ user: GeneralUser }>) {
    const { data } = useContext(StorageContext)
    const [client, setClient] = useState<MessagingClient | null>(null)
    const [msgUpdate, setUpdate] = useState(0)

    const { clients } = window
    useEffect(() => {
        if (!clients)
            return

        const c = clients.get(user.onionHostname);
        setClient(c)

        const listener = () => {
            setUpdate(Math.random())
        }

        c.addListener("on_update", listener)

        return () => {
            c.removeListener("on_update", listener)
        }
    }, [user])

    if (data === null) {
        return <Text>Loading storage data...</Text>
    }

    return <ChatContext.Provider value={{
        client,
        msgUpdate
    }}>
        {children}
    </ChatContext.Provider>
}
import React, { useContext, useState } from 'react'
import { ChatMessage } from '../../../bindings/rs/ChatMessage'
import { StorageContext } from '../storage/StorageProvider'
import { GeneralUser } from '../../../bindings/ws/client/types'
import { Text } from '@chakra-ui/react'
import { StorageData } from '../../../bindings/rs/StorageData'


export type AdditionalInfo = {
    failed: boolean,
    sending: boolean
}

export type ChatContextState = {
    messages: ChatMessage[],
    additional: Map<number, AdditionalInfo>,
    addMessage: (msg: ChatMessage) => void,
    setAdditional: (id: number, additional: AdditionalInfo) => void
}

export const ChatContext = React.createContext<ChatContextState>({} as ChatContextState)

export default function ChatProvider({ children, user }: React.PropsWithChildren<{ user: GeneralUser }>) {
    const { data, setData } = useContext(StorageContext)
    const [additional, setAdditional] = useState(new Map<number, AdditionalInfo>())

    if (data === null) {
        return <Text>Loading storage data...</Text>
    }

    const addr = user.onionHostname;

    const chat = data.chats[addr]

    return <ChatContext.Provider value={{
        messages: chat?.messages ?? [],
        addMessage: msg => {
            setData(prev => {
                const cloned: StorageData = JSON.parse(JSON.stringify(prev))
                if (!cloned.chats[addr])
                    cloned.chats[addr] = {
                        messages: [],
                        receiver_onion: addr,
                        nickname: null
                    }

                console.log("Msg is", cloned.chats[addr].messages)
                cloned.chats[addr].messages.push(msg)
                return cloned;
            })
        },

        additional: additional,
        setAdditional: (id, additional) => {
            setAdditional(prev => {
                const cloned = new Map(prev)

                cloned.set(id, additional)
                return cloned
            })
        }
    }}>
        {children}
    </ChatContext.Provider>
}
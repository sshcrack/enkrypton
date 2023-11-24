import { Flex, FlexProps, Input, InputGroup, InputLeftAddon, Text } from '@chakra-ui/react';
import { useContext, useEffect, useRef, useState } from 'react';
import 'react-chat-elements/dist/main.css';
import { MainContext } from '../MainProvider';
import ChatProvider, { ChatContext } from './ChatProvider';
import Messages from './Messages';
import SendButton from './SendButton';
import './index.scss';
import StatusScreen from './status/StatusScreen';



export default function Chat({ children, ...props }: FlexProps) {
    const { active } = useContext(MainContext)

    if (!active)
        return;


    return <ChatProvider user={active}>
        <ChatInner {...props}>
            {children}
        </ChatInner>
    </ChatProvider>
}

export function ChatInner(props: FlexProps) {
    const { active } = useContext(MainContext)
    const { client, msgUpdate } = useContext(ChatContext)

    const [update, setUpdate] = useState(0)


    const chatFieldRef = useRef<HTMLDivElement>(null)
    useEffect(() => {
        if (!chatFieldRef.current)
            return

        const curr = chatFieldRef.current;
        curr.scrollTop = curr.scrollHeight;
    }, [chatFieldRef, msgUpdate])


    // Just here to update the seconds
    useEffect(() => {
        setTimeout(() => {
            setUpdate(update + 1)
        }, 10 * 1000)
    }, [update])

    // Just waiting for it to load
    if (!client)
        return <Text>Loading...</Text>

    if (client.status !== 'Connected')
        return <Flex w="100%" h="100%" flexDir="column">
            <InputGroup>
                <InputLeftAddon children='Hostname' />
                <Input value={active?.onionHostname} isReadOnly />
            </InputGroup>
            <StatusScreen client={client} />
        </Flex>


    return <Flex w='100%' h='100%' flexDir='column' p='5'
        {...props}
    >
        <Flex w='100%' h='100%' flex='1' flexDir='column' overflowX='auto' ref={chatFieldRef} pb='3'>
            <InputGroup>
                <InputLeftAddon children='Hostname' />
                <Input value={active?.onionHostname} isReadOnly />
            </InputGroup>
            <Messages />
        </Flex>
        <Flex w='100%'>
            <SendButton client={client} />
        </Flex>
    </Flex>
}
import { Flex, FlexProps, Input, InputGroup, InputLeftAddon, Text } from '@chakra-ui/react';
import { useContext, useEffect, useRef, useState } from 'react';
import 'react-chat-elements/dist/main.css';
import { MainContext } from '../MainProvider';
import ChatProvider, { ChatContext } from './ChatProvider';
import Messages from './Messages';
import SendForm from './SendButton';
import './index.scss';
import StatusScreen from './status/StatusScreen';
import ConnectButton from './ConnectButton';


/**
 * Displays the chat of the currently active client (fetched from MainContext).
 * @param props Just the flexbox props
 */
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

function ChatInner(props: FlexProps) {
    // The currently focused client (onion hostname of it)
    const { active } = useContext(MainContext)
    // The client itself and the messageUpdate number to cause a rerender / refetch
    const { client, msgUpdate } = useContext(ChatContext)

    const [update, setUpdate] = useState(0)
    const [pressedConnected, setPressedConnect] = useState(false)

    // Just scrolling to the bottom when a new message is received
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

    // Waiting for the client to finally connect
    if (!client)
        return <Text>Loading...</Text>



    return <Flex w='100%' h='100%' flexDir='column' p='5'
        {...props}
    >
        <Flex w='100%' h='100%' flex='1' flexDir='column' overflowX='auto' ref={chatFieldRef} pb='3'>
            <InputGroup>
                <InputLeftAddon children='Hostname' />
                <Input value={active?.onionHostname} isReadOnly />
            </InputGroup>
            {(client?.status && client.status !== "Connected") ? <StatusScreen client={client} /> : <Messages />}
        </Flex>
        <Flex w='100%' >
            { !client?.status || client.status !== "Connected" ? <ConnectButton pressedConnect={pressedConnected} setPressedConnect={setPressedConnect} /> : <SendForm /> }
        </Flex>
    </Flex>
}
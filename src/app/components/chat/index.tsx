import { Flex, FlexProps, Input, InputGroup, InputLeftAddon, Text } from '@chakra-ui/react';
import { useContext, useEffect, useState, useRef } from 'react';
import 'react-chat-elements/dist/main.css';
import ws from '../../../bindings/ws';
import { MainContext } from '../MainProvider';
import './index.scss';
import ChatProvider, { ChatContext } from './ChatProvider';
import MessagingClient from '../../../bindings/ws/client';
import Messages from './Messages';
import SendButton from './SendButton';
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
    const [client, setClient] = useState(null as null | MessagingClient)

    const { addMessage } = useContext(ChatContext)

    const [update, setUpdate] = useState(0)
    const [focus, setFocus] = useState(0)


    const chatFieldRef = useRef<HTMLDivElement>(null)
    useEffect(() => {
        if (!chatFieldRef.current)
            return

        const curr = chatFieldRef.current;
        curr.scrollTop = curr.scrollHeight;
    }, [chatFieldRef, focus])

    useEffect(() => {
        if (!active)
            return;

        console.log("Getting client", active)
        setClient(ws.get(active.onionHostname))
    }, [active])


    // Just here to update the seconds
    useEffect(() => {
        setTimeout(() => {
            setUpdate(update + 1)
        }, 10 * 1000)
    }, [update])


    useEffect(() => {
        if (!active || client === null)
            return;


        const rec = (msg: string) => {
            console.log("received", msg)
            addMessage({ msg, self_sent: false, date: Date.now() })
            setFocus(Math.random())
        }

        client.on("on_receive", rec)
        return () => {
            client.removeListener("on_receive", rec)
        }
    }, [active, client, addMessage])

    // Just waiting for it to load
    if (!active || !client)
        return <Text>Loading...</Text>

    if(client.status !== 'Connected')
        return <StatusScreen client={client}/>


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
            <SendButton client={client} setFocus={setFocus} />
        </Flex>
    </Flex>
}
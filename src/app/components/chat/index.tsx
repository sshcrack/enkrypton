import { Button, Flex, FlexProps, Input, InputGroup, InputLeftAddon, InputRightElement, Text } from '@chakra-ui/react';
import { useContext, useEffect, useState, useRef } from 'react';
import { MessageBox } from 'react-chat-elements';
import 'react-chat-elements/dist/main.css';
import ws from '../../../bindings/ws';
import { MainContext } from '../MainProvider';
import './index.scss';
import ChatProvider, { ChatContext } from './ChatProvider';
import MessagingClient from '../../../bindings/ws/client';
import RenderIfVisible from 'react-render-if-visible'

const ESTIMATED_ITEM_HEIGHT = 100


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

    const { messages, addMessage, additional, setAdditional } = useContext(ChatContext)

    const [msg, setMsg] = useState("")
    const [sending, setSending] = useState(false)
    const [update, setUpdate] = useState(0)
    const [focus, setFocus] = useState(0)


    const chatFieldRef = useRef<HTMLDivElement>(null)
    useEffect(() => {
        console.log("Curr update")
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
    if (!active)
        return <Text>Loading...</Text>

    const onSend = () => {
        if (msg.length === 0 || client === null)
            return;

        setSending(true)
        setMsg("")

        const id = Date.now()
        addMessage({ msg, self_sent: true, date: id })
        setAdditional(id, { failed: false, sending: true })
        setFocus(Math.random())

        client.send(msg)
            .then(() => {
                setAdditional(id, { failed: false, sending: false })
            })
            .catch(e => {
                setAdditional(id, { failed: true, sending: false })
                console.error("Failed to send message", e)
            })
            .finally(() => setSending(false))
    }

    return <Flex w='100%' h='100%' flexDir='column' p='5'
        {...props}
    >
        <Flex w='100%' h='100%' flex='1' flexDir='column' overflowX='auto' ref={chatFieldRef} pb='3'>
            <InputGroup>
                <InputLeftAddon children='Hostname' />
                <Input value={active?.onionHostname} isReadOnly />
            </InputGroup>
            {messages.map(({ msg, self_sent, date }, i) => {
                let { failed, sending } = additional.get(date) ?? { failed: false, sending: false }

                const msgComp = <MessageBox
                    position={self_sent ? "right" : "left"}
                    type={'text'}
                    key={i}
                    date={date}
                    focus={false}
                    forwarded={false}
                    id={date}
                    notch={failed}
                    removeButton={false}
                    replyButton={false}
                    retracted={failed}
                    statusTitle={failed ? "Failed to send" : undefined}
                    status={sending || failed ? "waiting" : "received"}
                    text={msg}
                    title={failed ? "Failed to send" : (self_sent ? "You" : "Other")}
                    titleColor={failed ? "red" : 'white'}
                />

                return <RenderIfVisible defaultHeight={ESTIMATED_ITEM_HEIGHT}>
                    {msgComp}
                </RenderIfVisible>
            })}
        </Flex>
        <Flex w='100%'>
            <InputGroup size='md'>
                <Input pr='4.5rem' type='text' placeholder='Enter Message' value={msg} onChange={e => setMsg(e.target.value)} onKeyUp={(e) => e.key === "Enter" && !sending && onSend()}
                />
                <InputRightElement width='4.5rem'>
                    <Button colorScheme='green' borderLeftRadius='0' onClick={() => onSend()} isLoading={sending}
                    >Send</Button>
                </InputRightElement>
            </InputGroup>
        </Flex>
    </Flex>
}
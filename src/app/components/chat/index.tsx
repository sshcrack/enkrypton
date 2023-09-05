import { Button, Flex, FlexProps, Input, InputGroup, InputRightElement, Text } from '@chakra-ui/react';
import { useContext, useEffect, useState } from 'react';
import { MessageBox } from 'react-chat-elements';
import 'react-chat-elements/dist/main.css';
import ws from '../../../bindings/ws';
import { MainContext } from '../MainProvider';
import './index.scss';

//TODO actually do chat thing this is ust a demo
type Message = {
    /**
     * Weither the user has sent this message
     */
    selfSent: boolean,
    msg: string,
    date: number
}

export default function Chat(props: FlexProps) {
    const { active } = useContext(MainContext)
    const [msg, setMsg] = useState("")
    const [sending, setSending] = useState(false)
    const [messages, setMessages] = useState([] as Message[])


    useEffect(() => {
        if (!active)
            return;

        console.log("Getting client", active)
        const temp = ws.get(active.onionHostname)
        const rec = (msg: string) => {
            console.log("received", msg)
            setMessages([...messages, { msg, selfSent: false, date: Date.now() }])
        }

        temp.on("on_receive", rec)
        return () => {
            temp.removeListener("on_receive", rec)
        }
    }, [active, messages])

    // Just waiting for it to load
    if (!active)
        return <Text>Loading...</Text>

    const client = ws.get(active.onionHostname)

    const onSend = () => {
        if (msg.length === 0)
            return;

        setSending(true)
        setMsg("")
        client.send(msg)
            .then(() => {
                setMessages([...messages, { msg, selfSent: true, date: Date.now() }])
            })
            .finally(() => setSending(false))
    }

    return <Flex
        w='100%'
        h='100%'
        flexDir='column'
        p='5'
        {...props}
    >
        <Flex w='100%' h='100%' flex='1' flexDir='column' overflowX='auto'>
            {messages.map(({ msg, selfSent, date }, i) => {
                console.log("processing", msg, selfSent)
                return <MessageBox
                    position={selfSent ? "right" : "left"}
                    type={'text'}
                    date={date}
                    focus={messages.length - 1 === i}
                    forwarded={false}
                    id={i}
                    notch={false}
                    removeButton={false}
                    replyButton={false}
                    retracted={false}
                    status='received'
                    text={msg}
                    title={selfSent ? "You" : "Other"}
                    titleColor='white'
                />
            })}
        </Flex>
        <Flex w='100%'>
            <InputGroup size='md'>
                <Input
                    pr='4.5rem'
                    type='text'
                    placeholder='Enter Message'
                    value={msg}
                    onChange={e => setMsg(e.target.value)}
                    onKeyUp={(e) => e.key === "Enter" && onSend()}
                />
                <InputRightElement width='4.5rem'>
                    <Button
                        colorScheme='green'
                        borderLeftRadius='0'
                        onClick={() => onSend()}
                        isLoading={sending}
                    >Send</Button>
                </InputRightElement>
            </InputGroup>
        </Flex>
    </Flex>
}
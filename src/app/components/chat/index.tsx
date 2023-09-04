import { Button, Flex, FlexProps, Input, InputGroup, InputRightElement, Text, useToast } from '@chakra-ui/react';
import { useContext, useEffect, useState } from 'react';
import { MainContext } from '../MainProvider';
import ws from '../../../bindings/ws';

export default function Chat(props: FlexProps) {
    const { active } = useContext(MainContext)
    const [msg, setMsg] = useState("")
    const toast = useToast()


    useEffect(() => {
        if (!active)
            return;

        const temp = ws.get(active.onionHostname)
        const rec = (msg: string) => {
            toast({ title: "Received msg", description: msg, status: "info" })
        }

        temp.on("on_receive", rec)
        return () => {
            temp.removeListener("on_receive", rec)
        }
    }, [active])

    // Just waiting for it to load
    if (!active)
        return <Text>Loading...</Text>

    const client = ws.get(active.onionHostname)

    const onSend = () => {
        if (msg.length === 0)
            return;

        client.send(msg).then(() => toast({ description: "Message sent!", status: "success" }))
    }

    return <Flex
        w='100%'
        h='100%'
        flexDir='column'
        p='5'
        {...props}
    >
        <Flex w='100%' h='100%' flex='1'>

        </Flex>
        <Flex w='100%'>
            <InputGroup size='md'>
                <Input
                    pr='4.5rem'
                    type='text'
                    placeholder='Enter Message'
                    value={msg}
                    onChange={e => setMsg(e.target.value)}
                />
                <InputRightElement width='4.5rem'>
                    <Button
                        colorScheme='green'
                        borderLeftRadius='0'
                        onClick={() => onSend()}
                    >Send</Button>
                </InputRightElement>
            </InputGroup>
        </Flex>
    </Flex>
}
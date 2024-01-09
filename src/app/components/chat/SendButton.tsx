import { Button, Input, InputGroup, InputRightElement } from '@chakra-ui/react';
import { useContext, useState } from 'react';
import { ChatContext } from './ChatProvider';

/**
 * The input group to send a message.
 * @param _ no params
 */
export default function SendForm(_: {}) {
    // The client itself and the messageUpdate number to cause a rerender / refetch
    const { client } = useContext(ChatContext)
    const [msg, setMsg] = useState("")
    const [sending, setSending] = useState(false)

    const onSend = () => {
        if (msg.length === 0 || client === null)
            return;

        setSending(true)
        setMsg("")

        client.send(msg)
            .finally(() => setSending(false))
    }

    return <InputGroup size='md'>
        <Input pr='4.5rem' type='text' placeholder='Enter Message' value={msg} onChange={e => setMsg(e.target.value)} onKeyUp={(e) => e.key === "Enter" && !sending && onSend()}
        />
        <InputRightElement width='4.5rem'>
            <Button colorScheme='green' borderLeftRadius='0' onClick={() => onSend()} isLoading={sending || !client}
            >Send</Button>
        </InputRightElement>
    </InputGroup>
}
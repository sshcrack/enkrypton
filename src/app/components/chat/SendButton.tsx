import { Button, Input, InputGroup, InputRightElement } from '@chakra-ui/react';
import { useState } from 'react';
import MessagingClient from '../../../bindings/ws/client';

export type SendButtonProps = {
    client: MessagingClient | null
}

export default function SendButton({ client }: SendButtonProps) {
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
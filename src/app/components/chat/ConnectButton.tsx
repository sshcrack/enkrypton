import { Button, Flex } from '@chakra-ui/react';
import MessagingClient from '../../../bindings/ws/client';
import { ReactSetState } from '../../../tools/react';

export type SendButtonProps = {
    client: MessagingClient | null,
    setPressedConnect: ReactSetState<boolean>,
    pressedConnect: boolean
}

export default function ConnectButton({ client, pressedConnect, setPressedConnect }: SendButtonProps) {
    const onConnect = () => {
        if (client === null)
            return;

        setPressedConnect(true)
        setPressedConnect(true)
        client.connect()
            .finally(() => setPressedConnect(false))
    }

    return <Flex w='100%' justifyContent='center' alignItems='center'>
        <Button colorScheme='green' onClick={() => onConnect()} isLoading={pressedConnect || !client}
                >Connect to Host</Button>
    </Flex>
}
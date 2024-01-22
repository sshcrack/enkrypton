import { Button, Flex, useToast } from '@chakra-ui/react';
import { useContext } from 'react';
import { ReactSetState } from '../../../tools/react';
import { ChatContext } from './ChatProvider';

export type SendButtonProps = {
    /**
     * Sets whether the button is pressed or not.
     */
    setPressedConnect: ReactSetState<boolean>,
    /**
     * Whether the button is pressed or not.
     */
    pressedConnect: boolean
}

export default function ConnectButton({ pressedConnect, setPressedConnect }: SendButtonProps) {
    const { client } = useContext(ChatContext)
    const toast = useToast({})
    
    // The client itself and the messageUpdate number to cause a rerender / refetch
    const onConnect = () => {
        if (client === null)
            return toast({ title: "No client to connect to", status: "error" });

        setPressedConnect(true)
        // Just has to be twice to fire a re-render for sure
        setPressedConnect(true)
        client.connect()
            .catch(e => {
                console.error(e)
                toast({ title: "Could not connect to host", description: e, status: "error" })
            })
            .finally(() => setPressedConnect(false))
    }

    return <Flex w='100%' justifyContent='center' alignItems='center'>
        <Button
            colorScheme='green'
            onClick={() => onConnect()}
            isLoading={pressedConnect || !client}
        >Connect to Host</Button>
    </Flex>
}
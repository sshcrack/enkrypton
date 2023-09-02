import { Button, FormControl, FormErrorMessage, FormHelperText, FormLabel, Input, useToast } from '@chakra-ui/react'
import { useEffect, useState } from 'react';
import ws from '../../bindings/ws';
import tor from '../../bindings/tor';

export type ConnectFormProps = {
    onConnected: () => unknown
}

function isAddressValid(value: string) {
    //if (!value.endsWith(".onion") && !value.endsWith(".onion:80"))
    //    return false;

    return true;
}


const debugMode = true;

export default function ConnectForm({ onConnected }: ConnectFormProps) {
    const [connecting, setConnecting] = useState(false);
    const [input, setInput] = useState('')
    const toast = useToast()

    //@ts-ignore
    window.ws = ws;
    useEffect(() => {
        setConnecting(true)
        tor.get_hostname()
            .then(e => {
                setInput(`ws://${e}/ws/`)
            })
            .finally(() => setConnecting(false))
    }, [])

    useEffect(() => onSubmit(), [input])

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => setInput(e.target.value)
    const isError = !isAddressValid(input)
    const onSubmit = () => {
        console.log("On submit", input)
        if (isError)
            return;

        setConnecting(true)

        console.log("Conn")
        const client = ws.get(input)

        client.connect()
            .then(() => {
                console.log("Connected")
                onConnected()
            })
            .catch(e => toast({
                title: "Error connecting",
                description: e
            }))
            .finally(() => setConnecting(false))
    }

    return <FormControl>
        <FormLabel>Onion Address</FormLabel>
        <Input type='text' value={input} onChange={e => !debugMode && handleInputChange(e)} />
        {!isError ? (
            <FormHelperText>Enter the address the client should connect to</FormHelperText>
        ) : (
            <FormErrorMessage>The address has to end with .onion</FormErrorMessage>
        )}
        <Button
            mt={4}
            colorScheme='teal'
            isLoading={connecting}
            type='submit'
            onClick={() => onSubmit()}
        >Connect</Button>
    </FormControl>
}
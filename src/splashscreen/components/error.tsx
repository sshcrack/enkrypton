import { Button, Flex, Heading, Text } from '@chakra-ui/react'
import { StartTorErrorPayload } from '../payloads/StartTorPayload'
import { invoke } from '@tauri-apps/api/tauri';
import { useEffect } from "react"
import { invokeWindowTauri } from '../../tools/tauri';
//import { window } from "@tauri-apps/api"

export type ErrorScreenProps = {
    error: StartTorErrorPayload
}

export default function ErrorScreen({ error }: ErrorScreenProps) {
    const logs = error?.logs;
    useEffect(() => {
        const window = "splashscreen"
        invokeWindowTauri(window, "setDecorations", true)
        invokeWindowTauri(window, "setMaximizable", true)
        invokeWindowTauri(window, "maximize")
    }, [])


    return <Flex
        w='100%'
        h='100%'
        flexDir='column'
        justifyContent='center'
        alignItems='center'
    >
        <Heading size='sm' pb='5'>Could not start tor</Heading>
        <Flex
            w='100%'
            h='60%'
            flexDir='column'
            justifyContent='space-between'
            alignItems='center'
        >
            <Flex flexDir='column' alignItems='center' justifyContent='center'>
                <Flex
                    w='90%'
                    flexDir='column'
                    alignItems='center'
                    justifyContent='center'
                    gap='3'
                    bg='blackAlpha.600'
                >
                    <Text>Exit code: {error.error_code}</Text>
                    <Text className="log-style" p='5'>{logs.join("\n")}</Text>
                </Flex>
            </Flex>

            <Button onClick={() => invoke("restart").then(() => console.log("Restarting..."))} size='xl' colorScheme='orange' padding='5'>Restart</Button>
        </Flex>
    </Flex>
}
import { Button, Flex, Heading, Text } from '@chakra-ui/react'
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from "react"
import { invokeWindowTauri } from '../../tools/tauri';
import { TorStartupErrorPayload } from '../../bindings/rs/TorStartupErrorPayload';
//import { window } from "@tauri-apps/api"

export type ErrorScreenProps = {
    error: TorStartupErrorPayload
}

// Exit code for tor when it's already running (so we know it must be killed with a restart of this app)
const ERROR_TOR_CODE = "No, it's still there.  Exiting."
/**
 * A page to show the tor error correctly and give the user a button to restart the app.
 * @param props The error payload that was given by the backend.
 */
export default function ErrorScreen({ error }: ErrorScreenProps) {
    const logs = error.logs;
    const should_kill = logs?.some(e => e.includes(ERROR_TOR_CODE)) ?? false
    const [isRestarting, setRestarting] = useState(false)

    useEffect(() => {
        // Maximize window to show the erorr correctly
        const window = "splashscreen"
        const p1 = invokeWindowTauri(window, "setDecorations", true)
        const p2 = invokeWindowTauri(window, "setMaximizable", true)
        const p3 = invokeWindowTauri(window, "maximize")
        Promise.all([p1, p2, p3]).catch(console.error)
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
                    <Text>Exit code: {error.error_code ?? "none"}</Text>
                    <Text className="log-style" p='5'>{logs?.join("\n") ?? error.message}</Text>
                </Flex>
            </Flex>

            <Button
            onClick={() => {console.log("Restarting...")
                invoke("restart", { killOldTor: should_kill }).then(() => console.log("Should be started already lol"))
                setRestarting(true)
            }}
            size='xl'
            colorScheme='orange'
            padding='5'
            isLoading={isRestarting}
            >Restart</Button>
        </Flex>
    </Flex>
}
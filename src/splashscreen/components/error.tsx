import { Button, Flex, Heading, Text } from '@chakra-ui/react'
import { invoke } from '@tauri-apps/api/tauri';
import { useEffect, useState } from "react"
import { invokeWindowTauri } from '../../tools/tauri';
import { TorStartupErrorPayload } from '../../bindings/rs/TorStartupErrorPayload';
//import { window } from "@tauri-apps/api"

export type ErrorScreenProps = {
    error: TorStartupErrorPayload
}

const ERROR_TOR_CODE = "No, it's still there.  Exiting."
export default function ErrorScreen({ error }: ErrorScreenProps) {
    const logs = error.logs;
    const should_kill = logs?.some(e => e.includes(ERROR_TOR_CODE)) ?? false
    const [isRestarting, setRestarting] = useState(false)

    useEffect(() => {
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
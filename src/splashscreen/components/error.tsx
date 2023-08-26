import { Flex, Heading, Text } from '@chakra-ui/react'
import { StartTorErrorPayload } from '../payloads/StartTorPayload'

export type ErrorScreenProps = {
    error: StartTorErrorPayload
}

export default function ErrorScreen({ error }: ErrorScreenProps) {
    const logs = error?.logs?.concat([])?.reverse()
        // not keeping the date
        .map(e => {
            let index = e.indexOf("[");
            if (index === -1)
                index = 0

            return e.substring(index)
        });


    return <Flex
        w='100%'
        h='100%'
        flexDir='column'
        justifyContent='center'
        alignItems='center'
    >
        <Heading size='sm'>Could not start tor</Heading>
        <Flex
            w='90%'
            flexDir='column'
            alignItems='center'
            justifyContent='center'
            gap='3'
        >
            <Text>Exit code: {error.error_code}</Text>
            <Text className="log-style">{logs.join("\n")}</Text>
        </Flex>
    </Flex>
}
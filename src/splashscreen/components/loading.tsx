import { Flex, Progress, Text } from '@chakra-ui/react'

export type LoadingScreenProps = {
    /**
     * The progress of the tor initialization.
     */
    progress: number,
    /**
     * The current status message from tor.
     */
    status: string
}

/**
 * Creates a loading screen to display tor progress
 * @param props Type of LoadingScreenProps
 */
export default function LoadingScreen({ status, progress }: LoadingScreenProps) {
    return <Flex w='100%' h='100%' flexDir='column' justifyContent='center' alignItems='center'>
        <Flex
            w='85%'
            h='100%'
            flexDir='column'
            justifyContent='center'
            alignItems='center'
            gap='5'
        >
            <Text>{status}</Text>
            <Progress w='100%' value={progress * 100} rounded='xl'/>
        </Flex>
    </Flex>
}
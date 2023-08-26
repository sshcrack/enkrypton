import { Flex, Progress, Text } from '@chakra-ui/react'

export type LoadingScreenProps = {
    progress: number,
    status: string
}

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
            <Progress w='100%' value={progress} rounded='xl'/>
        </Flex>
    </Flex>
}
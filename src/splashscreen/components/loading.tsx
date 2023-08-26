import { Flex, Progress, Text } from '@chakra-ui/react'

export type LoadingScreenProps = {
    progress: number,
    status: string
}

export default function LoadingScreen({ status, progress }: LoadingScreenProps) {
    
    console.log("l page")
    return <Flex w='100%' h='100%' flexDir='column'>
        <Flex
            w='85%'
            h='100%'
            flexDir='column'
            justifyContent='center'
            alignItems='center'
        >
            <Text>{status}</Text>
            <Progress value={progress} />
        </Flex>
    </Flex>
}
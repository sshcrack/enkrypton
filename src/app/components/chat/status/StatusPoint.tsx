import { Flex, Text } from '@chakra-ui/react'

export type StatusPointProps = React.PropsWithChildren<{
    label: string
}>

export default function StatusPoint({ children, label }: StatusPointProps) {
    return <Flex flexDir='column' justifyContent='center' alignItems='center' gap='1'>
        {children}
        <Flex height='var(--text-height)' justifyContent='center' alignItems='center'>
            <Text>{label}</Text>
        </Flex>
    </Flex>
}
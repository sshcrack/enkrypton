import { Flex, Text } from '@chakra-ui/react'
import React from "react";

export type StatusPointProps = React.PropsWithChildren<{
    /**
     * The label to display under the point.
     */
    label: string
}>

/**
 * A status point to represent the steps between connecting to a client.
 * @param props For more Doc look at StatusPointProps
 */
export default function StatusPoint({ children, label }: StatusPointProps) {
    return <Flex flexDir='column' justifyContent='center' alignItems='center' gap='1'>
        {children}
        <Flex height='var(--text-height)' justifyContent='center' alignItems='center'>
            <Text>{label}</Text>
        </Flex>
    </Flex>
}
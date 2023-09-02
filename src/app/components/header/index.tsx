import { Flex, Image, Text } from '@chakra-ui/react';

export default function Header() {
    return <Flex
        w='100%'
        h='10'
        bg='blackAlpha.500'
    >
        <Image src='/icon.svg' p='2' />
        <Flex flex='1' justifyContent='center' alignItems='center'>
            <Text>Enkrypton</Text>
        </Flex>
    </Flex>
}
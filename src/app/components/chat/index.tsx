import { useContext, useEffect } from 'react';
import { MainContext } from '../MainProvider';
import { Button, Flex, FlexProps, Input, InputGroup, InputRightElement } from '@chakra-ui/react';

export default function Chat(props: FlexProps) {
    const { active } = useContext(MainContext)

    useEffect(() => {

    }, [])

    return <Flex
        w='100%'
        h='100%'
        flexDir='column'
        p='5'
        {...props}
    >
        <Flex w='100%' h='100%' flex='1'>

        </Flex>
        <Flex w='100%'>
            <InputGroup size='md'>
                <Input
                    pr='4.5rem'
                    type='text'
                    placeholder='Enter Message'
                />
                <InputRightElement width='4.5rem'>
                    <Button
                        colorScheme='green'
                        borderLeftRadius='0'
                    >Send</Button>
                </InputRightElement>
            </InputGroup>
        </Flex>
    </Flex>
}
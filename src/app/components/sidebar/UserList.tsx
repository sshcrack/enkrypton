import { Button, Flex, FlexProps, Input, Modal, ModalBody, ModalCloseButton, ModalContent, ModalFooter, ModalHeader, ModalOverlay, Text, useDisclosure } from '@chakra-ui/react'
import { GeneralUser } from '../../../bindings/ws/client/types'
import UserSidebar from './User'
import { useContext, useEffect, useState } from "react"
import { MainContext } from '../MainProvider'
import { ReactSetState } from '../../../tools/react'

export type UserListProps = {
    /**
     * The users that should be displayed in the sidebar.
     */
    receivers: GeneralUser[],
    /**
     * The setter for the receivers.
     */
    setReceivers: ReactSetState<GeneralUser[]>
} & Omit<FlexProps, "children">

export default function UserList({ receivers, setReceivers, ...props }: UserListProps) {
    // Currently active user
    const { setActive, active } = useContext(MainContext);
    const [toAdd, setToAdd] = useState("")
    const { isOpen, onOpen, onClose } = useDisclosure()

    useEffect(() => {
        // Don't set the default receiver when there is already an active one
        if (active || receivers.length === 0)
            return

        setActive(receivers[0])
    }, [active, receivers])

    return <>
        <Flex
            h='100%'
            w='30%'
            flexDir='column'
            className='scroll'
            bg='blackAlpha.300'
            overflow='auto'
            {...props}
        >
            <Flex flexDir='column' w='100%' h='100%' flex='1' alignItems='center'>
                {
                    receivers.map(user => <UserSidebar key={user.onionHostname.toString()} flex='0' user={user} />)
                }
            </Flex>
            <Flex w='100%' justifyContent='center' pb='3'>
                <Button colorScheme='green' w='70%' onClick={() => onOpen()}>
                    Add user
                </Button>
            </Flex>
        </Flex>

        {/* Modal for adding users */}
        <Modal isOpen={isOpen} onClose={onClose}>
            <ModalOverlay />
            <ModalContent>
                <ModalHeader>Add User</ModalHeader>
                <ModalCloseButton />
                <ModalBody>
                    <Text>Which user do you want to add?</Text>
                    <Input value={toAdd} onChange={e => setToAdd(e.target.value)} />
                </ModalBody>

                <ModalFooter>
                    <Button colorScheme='red' mr={3} onClick={onClose} variant='outline'>
                        Cancel
                    </Button>
                    <Button colorScheme='green' variant='ghost' onClick={() => {
                        onClose()
                        setToAdd("")
                        if (receivers.some(e => e.onionHostname === toAdd))
                            return

                        setReceivers([...receivers, { nickname: "Unknown", onionHostname: toAdd }])
                    }}>Add User</Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    </>
}
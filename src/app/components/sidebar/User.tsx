import { Flex, FlexProps, Text } from '@chakra-ui/react'
import { GeneralUser } from '../../../bindings/ws/client/types'
import UserAvatar from './UserAvatar'
import { useContext } from "react"
import { MainContext } from '../MainProvider'

export type ReceiverSidebarProps = {
    /**
     * The user that should be represented by this sidebar.
     */
    user: GeneralUser
} & FlexProps

/**
 * Dispalys a sidebar item for a user.
 * @param props
 */
export default function UserSidebar({ user, ...props }: ReceiverSidebarProps) {
    const { setActive, active } = useContext(MainContext)

    // checking if current item is active
    const isActive = user.onionHostname === active?.onionHostname
    const { onionHostname, nickname } = user

    return <Flex
        w='100%'
        alignItems='center'
        height='128px'
        p='3'
        bg={isActive ? "whiteAlpha.200" : ""}
        transition='.2s ease-in-out background'
        _hover={{ bg: isActive ? "" : 'blackAlpha.500' }}
        onClick={() => setActive(user)}
        {...props}
    >
        <UserAvatar seed={onionHostname} />
        <Text textOverflow='ellipsis' p='3' w='100%' whiteSpace='nowrap' overflow='hidden'>{nickname ?? onionHostname}</Text>
    </Flex>
}
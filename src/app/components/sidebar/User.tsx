import { Flex, Text } from '@chakra-ui/react'
import MessagingClient from '../../../bindings/ws/client'
import UserAvatar from './UserAvatar'

export type ReceiverSidebarProps = {
    addr: URL,
    nickname?: string,
    client?: MessagingClient
}

export default function UserSidebar({ addr, client, nickname }: ReceiverSidebarProps) {
    return <Flex w='100%'>
        <UserAvatar seed={addr.hostname} />
        <Text display='none' textOverflow='ellipsis' p='3'>{addr.hostname}</Text>
    </Flex>
}
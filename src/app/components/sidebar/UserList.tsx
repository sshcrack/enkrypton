import { Flex, FlexProps } from '@chakra-ui/react'
import { GeneralUser } from '../../../bindings/ws/client/types'
import UserSidebar from './User'
import { useContext, useEffect } from "react"
import { MainContext } from '../MainProvider'

export type UserListProps = {
    receivers: GeneralUser[]
} & Omit<FlexProps, "children">

export default function UserList({ receivers, ...props }: UserListProps) {
    const { setActive, active } = useContext(MainContext);

    useEffect(() => {
        console.log(receivers, active)
        if (active || receivers.length === 0)
            return

        console.log("Setting active")
        setActive(receivers[0])
    }, [active, receivers])

    console.log("Active is", active)
    return <Flex
        h='100%'
        w='30%'
        flexDir='column'
        className='scroll'
        bg='blackAlpha.300'
        overflow='auto'
        {...props}
    >
        {
            receivers.map(user => <UserSidebar key={user.onionHostname.toString()} flex='0' user={user} />)
        }
    </Flex>
}
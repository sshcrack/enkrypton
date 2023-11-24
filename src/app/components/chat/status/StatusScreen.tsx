import { Flex, Heading } from '@chakra-ui/react';
import { useEffect, useState } from "react";
import { BsPersonFillGear } from "react-icons/bs";
import { RiComputerLine, RiPassValidFill } from "react-icons/ri";
import { WsClientStatus } from '../../../../bindings/rs/WsClientStatus';
import MessagingClient from '../../../../bindings/ws/client';
import StatusLineBetween from './StatusLineBetween';
import StatusPoint from './StatusPoint';

export type StatusScreenProps = {
    client: MessagingClient
}

export default function StatusScreen({ client }: StatusScreenProps) {
    const [status, setStatus] = useState<WsClientStatus | null>(client.status)

    useEffect(() => {
        const l = (s: WsClientStatus) => {
            console.log("Status change")
            setStatus(s)
        }

        client.addListener("on_status_change", l)
        return () => {
            client.removeListener("on_status_change", l)
        }
    }, [client])


    const failed = status == "Disconnected"
    const proxyDone = (status && status != "ConnectingProxy") as boolean
    const hostDone = (proxyDone && status != "ConnectingHost") as boolean
    const identityDone = (hostDone && status != "WaitingIdentity") as boolean

    const icoStyle = { width: "2.5em", height: "2.5em" };
    return <Flex w='100%' h='100%' justifyContent='center' flexDir='column' alignItems='center' p='6' style={{ "--text-height": "1.5em" } as any}>
        <Flex h='20%' transform='translateY(-20%)' flexDir='column' gap='5'>
            {failed && <Heading>Failed to connect.</Heading>}
            {status && !failed && <Heading>Connecting...</Heading>}
        </Flex>
        <Flex w='100%' justifyContent='space-evenly' gap='1' transform='translateY(-20%)' opacity={status == null ? "0" : "1"}>
            <StatusPoint label='Your Computer'>
                <RiComputerLine style={icoStyle} />
            </StatusPoint>

            <StatusLineBetween animate={!status || status == 'ConnectingProxy'} isDone={proxyDone} isFailed={failed} />

            <StatusPoint label='Tor Network'>
                <img src="/tor.svg" style={icoStyle} />
            </StatusPoint>

            <StatusLineBetween animate={proxyDone && status == 'ConnectingHost'} isDone={hostDone} isFailed={failed} />
            <StatusPoint label='Receiver'>
                <BsPersonFillGear style={icoStyle} />
            </StatusPoint>

            <StatusLineBetween animate={hostDone && status == 'WaitingIdentity'} isDone={identityDone} isFailed={failed} />
            <StatusPoint label='Identity check'>
                <RiPassValidFill style={icoStyle} />
            </StatusPoint>
        </Flex>
    </Flex>
}
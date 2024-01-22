// noinspection JSUnusedGlobalSymbols

import { useEffect, useState } from 'react';
import { Text } from '@chakra-ui/react';
import tor from '../../bindings/tor';

/**
 * A component to display the hostname of the tor client.
 */
export default function Hostname() {
    const [hostname, setHostname] = useState<string | null>(null)

    useEffect(() => {
        tor.get_hostname().then(e => setHostname(e))
    }, [])

    return <Text>Hostname: {hostname}</Text>
}
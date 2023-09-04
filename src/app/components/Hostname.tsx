import { useEffect, useState } from 'react';
import { Text } from '@chakra-ui/react';
import tor from '../../bindings/tor';

export default function Hostname() {
    const [hostname, setHostname] = useState<string | null>(null)

    useEffect(() => {
        tor.get_hostname().then(e => setHostname(e))
    }, [])

    return <Text>Hostname: {hostname}</Text>
}
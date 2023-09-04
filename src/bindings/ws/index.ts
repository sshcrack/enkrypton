import { ClientMap } from './client/map';

if (!window.clients)
    window.clients = new ClientMap();

const ws = {
    get: (onionHost: string) => window.clients.get(onionHost)
}

export default ws;
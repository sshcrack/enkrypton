import { ClientMap } from './client/map';

if (!window.clients)
    window.clients = new ClientMap();

const ws = {
    get: (onionAddr: string) => window.clients.get(onionAddr)
}

export default ws;
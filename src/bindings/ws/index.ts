import { listen, Event } from "@tauri-apps/api/event"
import { ClientMap } from './client/map';
import { WsClientUpdate } from '../rs/WsClientUpdate';

if (!window.clients)
    window.clients = new ClientMap();



type Func = (payload: WsClientUpdate) => unknown;
const listeners: Func[] = [];

const ws = {
    get: (onionHost: string) => window.clients.get(onionHost),
    addClientUpdateListener: (callback: (payload: WsClientUpdate) => unknown) => {
        listeners.push(callback)

        return () => {
            const index = listeners.indexOf(callback)
            if (index === -1)
                return console.error("Could not remove manual listener")

            listeners.splice(index, 1)
        }
    }
}


listen("ws_client_update", ({ payload }: Event<WsClientUpdate>) => {
    const { hostname, status } = payload;
    if (status === 'DISCONNECTED' && window.clients.has(hostname)) {
        const client = ws.get(hostname);
        client.destroy()

        window.clients.delete(hostname)
        return;

    }

    console.log("Received update listen ->", hostname)
    // Constructing the client
    ws.get(hostname)
})

export default ws;
import { listen, Event } from "@tauri-apps/api/event"
import { ClientMap } from './client/map';
import { WsClientUpdatePayload } from '../rs/WsClientUpdatePayload';
import { WsMessageStatusPayload } from '../rs/WsMessageStatusPayload';

if (!window.clients) {
    console.log("New client map")
    window.clients = new ClientMap();
}



type Func = (payload: WsClientUpdatePayload) => unknown;
const listeners: Func[] = [];

/**
 * All functions that are related to real time chat and to communicate with the rust backend.
 */
const ws = {
    /**
     * Gets the client for the given onion hostname.
     * @param onionHost The onion hostname of the client.
     * @returns The client for the given onion hostname.
     */
    get: (onionHost: string) => window.clients.get(onionHost),
    /**
     * Adds the given function as a callback which is called when the client connection status was updated.
     * @returns the function to remove the listener.
     */
    addClientUpdateListener: (callback: (payload: WsClientUpdatePayload) => unknown) => {
        listeners.push(callback)

        return () => {
            const index = listeners.indexOf(callback)
            if (index === -1)
                return console.error("Could not remove manual listener")

            listeners.splice(index, 1)
        }
    }
}


// Well, listens for the client update event and notifies listeners.
listen("ws_client_update", ({ payload }: Event<WsClientUpdatePayload>) => {
    listeners.map(l => l(payload))
    console.log("Received update", payload)
    const { hostname, status } = payload;
    if (status === 'Disconnected' && window.clients.has(hostname)) {
        const client = ws.get(hostname);
        client.destroy()

        window.clients.delete(hostname)
        return;

    }

    console.log("Received update listen ->", hostname)
    // Constructing the client
    const c = ws.get(hostname)
    c.status = status
}).catch(console.error)

listen("ws_msg_update", ({ payload: { hostname, date, status } }: Event<WsMessageStatusPayload>) => {
    const c = ws.get(hostname)

    console.log("Updating messages...")
    c.updateMsg(date, status).catch(console.error)
}).catch(console.error)

export default ws;
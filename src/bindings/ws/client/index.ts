import { invoke } from '@tauri-apps/api';
import { listen, Event, UnlistenFn } from "@tauri-apps/api/event"
import EventEmitter from "events"
import TypedEmitter from "typed-emitter"
import { WsMessagePayload } from '../../rs/WsMessagePayload';


export type MessagingClientEvents = {
    on_receive: (message: string) => unknown
}

export default class MessagingClient extends (EventEmitter as unknown as new () => TypedEmitter<MessagingClientEvents>) {
    private readonly onionAddr: string;
    private unlisten: UnlistenFn = () => { };

    constructor(onionAddr: string) {
        super()
        this.onionAddr = onionAddr;
    }

    public async destroy() {
        this.unlisten()
    }

    public async connect() {
        await invoke("ws_connect", { onionAddr: this.onionAddr })
        const { hostname } = new URL(this.onionAddr);

        this.unlisten = await listen(`msg-${hostname}`, ({ payload }: Event<WsMessagePayload>) => {
            this.emit("on_receive", payload.message)
        });
    }

    public async send(msg: string) {
        return invoke("ws_send", { onionAddr: this.onionAddr, msg });
    }
}
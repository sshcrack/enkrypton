import { invoke } from '@tauri-apps/api';
import { listen, Event, UnlistenFn } from "@tauri-apps/api/event"
import EventEmitter from "events"
import TypedEmitter from "typed-emitter"
import { WsMessagePayload } from '../../rs/WsMessagePayload';


export type MessagingClientEvents = {
    on_receive: (message: string) => unknown
}

export default class MessagingClient extends (EventEmitter as unknown as new () => TypedEmitter<MessagingClientEvents>) {
    private readonly onionHostname: string;
    private unlisten: UnlistenFn = () => { };

    constructor(onionHostname: string) {
        super()
        this.onionHostname = onionHostname;
    }

    public async destroy() {
        this.unlisten()
    }

    public async connect() {
        await invoke("ws_connect", { onionHostname: this.onionHostname })
        this.unlisten = await listen(`msg-${this.onionHostname}`, ({ payload }: Event<WsMessagePayload>) => {
            this.emit("on_receive", payload.message)
        });
    }

    public async send(msg: string) {
        return invoke("ws_send", { onionHostname: this.onionHostname, msg });
    }
}
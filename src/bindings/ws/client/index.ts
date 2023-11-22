import { invoke } from '@tauri-apps/api';
import { listen, Event, UnlistenFn } from "@tauri-apps/api/event"
import EventEmitter from "events"
import TypedEmitter from "typed-emitter"
import { WsMessagePayload } from '../../rs/WsMessagePayload';
import { WsClientStatus } from '../../rs/WsClientStatus';


export type MessagingClientEvents = {
    on_receive: (message: string) => unknown,
    on_status_change: (status: WsClientStatus) => unknown
}

export default class MessagingClient extends (EventEmitter as unknown as new () => TypedEmitter<MessagingClientEvents>) {
    private readonly onionHostname: string;
    private _status: WsClientStatus | null = null;
    private unlisten: UnlistenFn = () => { };

    constructor(onionHostname: string) {
        super()
        this.onionHostname = onionHostname;

        const msgEvent = `msg-${this.onionHostname}`

        console.log("Listening on", msgEvent)
        listen(msgEvent, ({ payload }: Event<WsMessagePayload>) => {
            console.log("received msg", payload.message, "on", this.onionHostname)
            this.emit("on_receive", payload.message)
        }).then(e => this.unlisten = e);
    }

    public async destroy() {
        this.unlisten()
    }

    public async connect() {
        await invoke("ws_connect", { onionHostname: this.onionHostname })
    }

    public async send(msg: string) {
        return invoke("ws_send", { onionHostname: this.onionHostname, msg });
    }

    public set status(status: WsClientStatus) {
        this._status = status;
        console.log("Received update status", status, "for", this.onionHostname)
        this.emit("on_status_change", status)
    }

    public get status(): WsClientStatus | null {
        return this._status
    }
}
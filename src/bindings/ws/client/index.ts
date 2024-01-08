import { invoke } from '@tauri-apps/api';
import { listen, Event, UnlistenFn } from "@tauri-apps/api/event"
import EventEmitter from "events"
import TypedEmitter from "typed-emitter"
import { WsMessagePayload } from '../../rs/WsMessagePayload';
import { WsClientStatus } from '../../rs/WsClientStatus';
import { WsMessageStatus } from '../../rs/WsMessageStatus';
import { ChatMessage } from '../../rs/ChatMessage';
import storage from '../../storage';

export type MessagingClientEvents = {
    on_receive: (message: string) => unknown,
    on_status_change: (status: WsClientStatus) => unknown,
    on_update: (date: number, status: WsMessageStatus) => unknown
}

export default class MessagingClient extends (EventEmitter as unknown as new () => TypedEmitter<MessagingClientEvents>) {
    private readonly onionHostname: string;
    private _status: WsClientStatus | null = null;
    private unlisten: UnlistenFn = () => { };

    private _messages: ChatMessage[] = [];

    constructor(onionHostname: string) {
        super()
        this.onionHostname = onionHostname;

        const msgEvent = `msg-${this.onionHostname}`

        console.log("Listening on", msgEvent)
        listen(msgEvent, ({ payload }: Event<WsMessagePayload>) => {
            console.log("received msg", payload.message, "on", this.onionHostname)
            this.emit("on_receive", payload.message)
        }).then(e => this.unlisten = e);

        this.updateMsg(-1, "Success").then(() => console.log("Updated msg"))
    }

    public destroy() {
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


    public messages(): ChatMessage[] {
        return this._messages as ChatMessage[];
    }

    public async updateMsg(date: number, status: WsMessageStatus) {
        console.log("Updating msg", date, status, this.onionHostname)
        let shouldUpdate = this._messages === null;
        if (this._messages !== null) {
            const msgIndex = this._messages.findIndex(m => m.date === date)
            if (msgIndex === -1) {
                shouldUpdate = true;
            } else {
                this._messages[msgIndex].status = status
            }
        }

        if (shouldUpdate) {
            const chat = (await storage.get()).chats[this.onionHostname];
            this._messages = chat?.messages ?? []
        }

        this.emit("on_update", date, status)
    }
}
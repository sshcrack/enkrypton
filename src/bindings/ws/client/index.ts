import { invoke } from '@tauri-apps/api/core';
import { listen, Event, UnlistenFn } from "@tauri-apps/api/event"
import EventEmitter from "events"
import TypedEmitter from "typed-emitter"
import { WsMessagePayload } from '../../rs/WsMessagePayload';
import { WsClientStatus } from '../../rs/WsClientStatus';
import { WsMessageStatus } from '../../rs/WsMessageStatus';
import { ChatMessage } from '../../rs/ChatMessage';
import storage from '../../storage';

/**
 * Events emitted by the messaging client
 */
export type MessagingClientEvents = {
    on_receive: (message: string) => unknown,
    on_status_change: (status: WsClientStatus) => unknown,
    on_update: (date: number, status: WsMessageStatus) => unknown
}

/**
 * Messaging client for a single onion hostname communicating with the rust backend.
 */
export default class MessagingClient extends (EventEmitter as unknown as new () => TypedEmitter<MessagingClientEvents>) {
    /** Hostname of this client. */
    private readonly onionHostname: string;
    /** Current connection status of this client. */
    private _status: WsClientStatus | null = null;
    /** Function to unlisten for events. */
    private unlisten: UnlistenFn = () => { };

    /** Messages of this client. */
    private _messages: ChatMessage[] = [];

    /**
     * Initializes the messaging client for the given onion hostname and listens for events.
     * @param onionHostname a valid onion hostname the client should listen to.
     */
    constructor(onionHostname: string) {
        super()
        this.onionHostname = onionHostname;

        const msgEvent = `msg-${this.onionHostname}`

        // Listens for the message event on te backend
        console.log("Listening on", msgEvent)
        listen(msgEvent, ({ payload }: Event<WsMessagePayload>) => {
            console.log("received msg", payload.message, "on", this.onionHostname)
            this.emit("on_receive", payload.message)
        }).then(e => this.unlisten = e);

        this.updateMsg(-1, "Success").then(() => console.log("Updated msg"))
    }

    /**
     * Destroys the messaging client and unlistens for events.
     */
    public destroy() {
        this.unlisten()
    }

    /**
     * Connects to the WebSocket server.
     * @returns A promise that resolves when the connection is established.
     */
    public async connect() {
        await invoke("ws_connect", { onionHostname: this.onionHostname })
    }

    /**
     * Sends a message to the WebSocket server.
     * @param msg The message to send.
     * @returns A promise that resolves when the message is sent.
     */
    public async send(msg: string) {
        return invoke("ws_send", { onionHostname: this.onionHostname, msg });
    }

    /**
     * Sets the status of the WsClient.
     * Emits the "on_status_change" event.
     * @param status - The new status for the WsClient.
     */
    public set status(status: WsClientStatus) {
        this._status = status;
        console.log("Received update status", status, "for", this.onionHostname)
        this.emit("on_status_change", status)
    }

    /**
     * Gets the status of the WebSocket client.
     * @returns The status of the WebSocket client, or null if the status is not set.
     */
    public get status(): WsClientStatus | null {
        return this._status
    }


    /**
     * Retrieves the chat messages.
     *
     * @returns An array of ChatMessage objects representing the chat messages.
     */
    public messages(): ChatMessage[] {
        return this._messages as ChatMessage[];
    }

    /**
     * Updates the status of a message.
     * @param date - The date the message was sent at.
     * @param status - The new status of the message.
     */
    public async updateMsg(date: number, status: WsMessageStatus) {
        console.log("Updating msg", date, status, this.onionHostname)
        let shouldUpdate = this._messages === null;

        // Check if there if the message is currently in storage, if not fetch them again from the backen
        // Or else just set the status of the message
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
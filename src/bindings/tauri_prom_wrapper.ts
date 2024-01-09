import { listen, UnlistenFn, Event } from '@tauri-apps/api/event'
import { StartTorPayload } from './rs/StartTorPayload';
import { TorStartupErrorPayload } from './rs/TorStartupErrorPayload';

type Event2Payload = {
    "tor_start": StartTorPayload,
    "tor_start_error": TorStartupErrorPayload,
    "splashscreen_closed": null
}

/**
 * Listens to a tauri event and calls the handler with the correct payload.
 * @param event The event name.
 * @param handler The handler to call.
 * @returns a function to remove the listener.
 */
export function listenSync<E extends keyof Event2Payload>(event: E, handler: (payload: Event<Event2Payload[E]>) => unknown) {
    let unlistenFunc: UnlistenFn = () => { }

    listen(event, (e) => {
        const typedE = e as Event<Event2Payload[E]>
        handler(typedE)
    }).then(e => unlistenFunc = e);

    return () => {
        unlistenFunc();
    };
}
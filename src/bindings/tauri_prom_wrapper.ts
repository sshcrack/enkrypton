import { listen, UnlistenFn, Event } from '@tauri-apps/api/event'
import { StartTorPayload } from './rs/StartTorPayload';
import { TorStartupErrorPayload } from './rs/TorStartupErrorPayload';

type Event2Payload = {
    "tor_start": StartTorPayload,
    "tor_start_error": TorStartupErrorPayload
}

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
import { ClientMap } from '../bindings/ws/client/map';

declare global {
    interface Window {
        clients: ClientMap
    }
}
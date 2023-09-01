import MessagingClient from '.';

export abstract class DefaultMap<K, V> extends Map<K, V> {
    abstract computeDefault(key: K): V;

    get(key: K): V {
        let res = super.get(key);
        if (res === undefined) {
            res = this.computeDefault(key)
            this.set(key, res)
        }

        return res
    }
}

export class ClientMap extends DefaultMap<string, MessagingClient> {
    computeDefault(key: string): MessagingClient {
        return new MessagingClient(key)
    }
}
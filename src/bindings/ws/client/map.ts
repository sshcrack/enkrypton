import MessagingClient from '.';

/**
 * Same as Map but with default values.
 * @template K Type of the keys in the map.
 * @template V Type of the values in the map.
 */
export abstract class DefaultMap<K, V> extends Map<K, V> {
    /**
     * Computes the default value for the given key.
     * @param key The key for which to compute the default value.
     * @returns The computed value.
     */
    abstract computeDefault(key: K): V;

    /**
     * Gets the value of the key or sets and returns the default value if the key does not exist.
     * @param key The key to retrieve the value for.
     * @returns The value associated with the specified key, or the default value if the key does not exist.
     */
    get(key: K): V {
        let res = super.get(key);
        if (res === undefined) {
            res = this.computeDefault(key)
            this.set(key, res)
        }

        return res
    }
}

/**
 * Just a client map which returns a new instance of MessagingClient if the key does not exist.
 */
export class ClientMap extends DefaultMap<string, MessagingClient> {
    computeDefault(key: string): MessagingClient {
        return new MessagingClient(key)
    }
}
/**
 * Represents a general user.
 */
export type GeneralUser = {
    /**
     * The nickname of the user.
     */
    nickname?: string,
    /**
     * The hostname of the user's onion service.
     */
    onionHostname: string
}
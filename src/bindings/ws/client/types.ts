/**
 * Represents just a general user with nickname and onion hostname
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
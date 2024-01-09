import { miniavs } from '@dicebear/collection';
import { createAvatar } from '@dicebear/core';
import { useMemo } from 'react';

export type UserAvatarProps = {
    /**
     * The seed to generate the avatar from.
     */
    seed: string
}

/**
 * Generates the avatar of a user from a given seed.
 * @param props
 */
export default function UserAvatar({ seed }: UserAvatarProps) {
    // Creates the avatar once from the given seed
    const avatar = useMemo(() => {
        return createAvatar(miniavs, {
            size: 64,
            seed
        }).toDataUriSync();
    }, []);

    return <img src={avatar} alt="Avatar" />
}
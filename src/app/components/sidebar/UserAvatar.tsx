import { miniavs } from '@dicebear/collection';
import { createAvatar } from '@dicebear/core';
import { useMemo } from 'react';

export type UserAvatarProps = {
    seed: string
}

export default function UserAvatar({ seed }: UserAvatarProps) {
    const avatar = useMemo(() => {
        return createAvatar(miniavs, {
            size: 64,
            seed
        }).toDataUriSync();
    }, []);

    return <img src={avatar} alt="Avatar" />
}
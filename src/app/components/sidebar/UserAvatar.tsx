import Avatar, { allOptions } from 'avataaars'
import { useMemo } from 'react'
import seedrandom from 'seedrandom'

export type UserAvatarProps = {
    seed: string
}

export default function UserAvatar({ seed }: UserAvatarProps) {
    const random = useMemo(() => seedrandom(seed), [])
    console.log(allOptions)
    return <Avatar
        style={{ width: '4em', aspectRatio: '1 / 1' }}
        avatarStyle='Circle'
        topType='LongHairMiaWallace'
        accessoriesType='Prescription02'
        hairColor='BrownDark'
        facialHairType='Blank'
        clotheType='Hoodie'
        clotheColor='PastelBlue'
        eyeType='Happy'
        eyebrowType='Default'
        mouthType='Smile'
        skinColor='Light'
    />
}
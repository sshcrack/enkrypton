import React, { useState } from 'react'
import { ReactSetState } from '../../tools/react'
import { GeneralUser } from '../../bindings/ws/client/types'

export type MainContextState = {
    setActive: ReactSetState<GeneralUser | null>,
    active: GeneralUser | null
}

export const MainContext = React.createContext<MainContextState>({
    setActive: () => { },
    active: null
})

export default function MainProvider({ children }: React.PropsWithChildren<{}>) {
    const [active, setActive] = useState<GeneralUser | null>(null)

    return <MainContext.Provider value={{ setActive, active }}>
        {children}
    </MainContext.Provider>
}
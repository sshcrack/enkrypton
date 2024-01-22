import React, { useState } from 'react'
import { ReactSetState } from '../../tools/react'
import { GeneralUser } from '../../bindings/ws/client/types'

export type MainContextState = {
    /**
     * Function to set the current user.
     */
    setActive: ReactSetState<GeneralUser | null>,
    /**
     * The currently active user.
     */
    active: GeneralUser | null
}

export const MainContext = React.createContext<MainContextState>({
    setActive: () => { },
    active: null
})

/**
 * Provides the active user and a setter for it.
 * @param props just contains the children to render
 */
export default function MainProvider({ children }: React.PropsWithChildren<{}>) {
    const [active, setActive] = useState<GeneralUser | null>(null)

    return <MainContext.Provider value={{ setActive, active }}>
        {children}
    </MainContext.Provider>
}
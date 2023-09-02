import { Flex, Text } from '@chakra-ui/react';
import { useState, useEffect } from "react"
import { GeneralUser } from '../bindings/ws/client/types';
import tor from '../bindings/tor';
import UserSidebar from './components/sidebar/User';
import Header from './components/header';

function App() {
  const [receivers, setReceivers] = useState<GeneralUser[]>([])

  useEffect(() => {
    tor.get_hostname()
      .then(e => {
        if (!e)
          return

        setReceivers([
          {
            nickname: "Self",
            onionAddr: new URL(`ws://${e}/ws/`)
          }
        ])
      })
  }, [])

  return <Flex w='100%' h='100%' flexDir='column'>
    <Header />
    <Flex w='100%' h='100%'>
      <Flex h='100%' w='30%' flexDir='column'>
        {
          receivers.map(({ nickname, onionAddr }) => <UserSidebar addr={onionAddr} nickname={nickname} />)
        }
      </Flex>
      <Flex flex='1' h='100%' w='70%'>
      </Flex>
    </Flex>
  </Flex>
}

export default App;

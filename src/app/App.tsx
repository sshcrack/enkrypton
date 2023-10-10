import { Flex } from '@chakra-ui/react';
import { useEffect, useState } from "react";
import tor from '../bindings/tor';
import { GeneralUser } from '../bindings/ws/client/types';
import Header from './components/header';
import UserList from './components/sidebar/UserList';
import MainProvider from './components/MainProvider';
import Chat from './components/chat';

function App() {
  const [receivers, setReceivers] = useState<GeneralUser[]>([])

  useEffect(() => {
    tor.get_hostname()
      .then(e => {
        if (!e)
          return

        const r: GeneralUser[] = [
          {
            nickname: "Self",
            onionHostname: e
          }
        ]

        setReceivers(r)
      })
  }, [])

  return <MainProvider>
    <Flex w='100%' h='100%' flexDir='column'>
      <Header />
      <Flex w='100%' h='100%'>
        <UserList receivers={receivers} setReceivers={setReceivers}/>
        <Chat flex='1' />
      </Flex>
    </Flex>
  </MainProvider>
}

export default App;

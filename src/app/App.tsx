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
  const [retry, setRetry] = useState(0)

  useEffect(() => {
    tor.get_hostname()
      .then(e => {
        const r: GeneralUser[] = [
          {
            nickname: "Self",
            onionHostname: e
          }
        ]

        setReceivers(r)
      }).catch(e => {
        setTimeout(() => setRetry(retry + 1), 100)
        return console.error("Failed to get hostname, retrying", e)
      })
  }, [])

  return <MainProvider>
    <Flex w='100%' h='100%' flexDir='column'>
      <Header />
      <Flex w='100%' h='100%'>
        <UserList receivers={receivers} setReceivers={setReceivers} />
        <Chat flex='1' />
      </Flex>
    </Flex>
  </MainProvider>
}

export default App;

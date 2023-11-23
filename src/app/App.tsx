import { Flex, Text } from '@chakra-ui/react';
import { useContext, useEffect, useState } from "react";
import tor from '../bindings/tor';
import { GeneralUser } from '../bindings/ws/client/types';
import Header from './components/header';
import UserList from './components/sidebar/UserList';
import MainProvider from './components/MainProvider';
import Chat from './components/chat';
import { StorageContext, StorageProvider } from './components/storage/StorageProvider';
import { listenSync } from '../bindings/tauri_prom_wrapper';
import ws from '../bindings/ws';

function App() {
  //TODO disable
  const [isReady, setReady] = useState(false);
  useEffect(() => {
    tor.splashscreen_closed().then(e => setReady(e))

    const unlisten = listenSync("splashscreen_closed", () => setReady(true));

    return () => unlisten()
  }, [])

  if (isReady)
    return <MainProvider>
      <StorageProvider>
        <InnerApp />
      </StorageProvider>
    </MainProvider>

  return <Text>Splashscreen is still shown...</Text>
}


function InnerApp() {
  const { data } = useContext(StorageContext)
  const [receivers, setReceivers] = useState<GeneralUser[]>([])
  const [hostname, setHostname] = useState<string | null>(null)
  const [retryHostname, setRetryHostname] = useState(0)
  const [update, setUpdate] = useState(0)

  //TODO do more efficiently and not that scuffed
  useEffect(() => ws.addClientUpdateListener((_) => {
    setUpdate(Math.random())
    console.log("Received payload, updating")
  }), [])

  useEffect(() => {
    tor.get_hostname().then(e => setHostname(e)).catch(e => {
      setTimeout(() => setRetryHostname(retryHostname + 1), 100)
      return console.error("Failed to get hostname, retrying", e)
    });
  }, [retryHostname])

  useEffect(() => {
    if (!data || !hostname)
      return;

    const saved_users: GeneralUser[] = Object.entries(data.chats)
      .map(([k, v]) => {
        return {
          nickname: k === hostname ? "Self" : v.nickname ?? undefined,
          onionHostname: k
        } as GeneralUser
      })

    if (!saved_users.some(savedUsr => savedUsr.nickname === "Self")) {
      saved_users.push({
        nickname: "Self",
        onionHostname: hostname
      })
    }


    setReceivers(saved_users)
  }, [hostname, data, update])

  return <Flex w='100%' h='100%' flexDir='column'>
    <Header />
    <Flex w='100%' h='100%'>
      <UserList receivers={receivers} setReceivers={setReceivers} />
      <Chat flex='1' />
    </Flex>
  </Flex>
}

export default App;

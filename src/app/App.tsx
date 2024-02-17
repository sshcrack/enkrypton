import { Button, Flex, Text } from '@chakra-ui/react';
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

/**
 * Main App component. Just a wrapper around some providers and the main part of the app, InnerApp.
 */
function App() {
  const [splashscreenClosed, setClosed] = useState(false);

  useEffect(() => {
    // Waiting for the splashscreen to close and to actually load stuff
    tor.is_splashscreen_closed().then(e => setClosed(e))

    const unlisten = listenSync("splashscreen_closed", () => setClosed(true));

    return () => unlisten()
  }, [])

  if (splashscreenClosed)
    return <MainProvider>
      <StorageProvider>
        <InnerApp />
      </StorageProvider>
    </MainProvider>

  // Waiting for splashscreen
  return <>
  <Text>Splashscreen is still shown...</Text>
  <Button colorScheme='green' onClick={() => setClosed(true)}>No it's not!</Button></>
}


/**
 * The actual app component. This is the root component of the application.
 */
function InnerApp() {
  // Current data of storage
  const { data } = useContext(StorageContext)

  const [receivers, setReceivers] = useState<GeneralUser[]>([])
  const [hostname, setHostname] = useState<string | null>(null)
  const [retryHostname, setRetryHostname] = useState(0)
  const [update, setUpdate] = useState(0)

  //TODO do more efficiently and not that scuffed
  useEffect(() => ws.addClientUpdateListener(() => {
    // Updating the whole app when a client update is received
    setUpdate(Math.random())
    console.log("Received payload, updating")
  }), [])

  useEffect(() => {
    // Trying to fetch current hostname and set it
    tor.get_hostname().then(e => setHostname(e)).catch(e => {
      setTimeout(() => setRetryHostname(retryHostname + 1), 100)
      return console.error("Failed to get hostname, retrying", e)
    });
  }, [retryHostname])

  useEffect(() => {
    if (!data || !hostname)
      return;

    // Getting all users from storage and setting them
    const saved_users: GeneralUser[] = Object.entries(data.chats)
      .map(([k, v]) => {
        return {
          nickname: k === hostname ? "Self" : v.nickname ?? undefined,
          onionHostname: k
        } as GeneralUser
      })

    // And adding ourselves for debugging purposes
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

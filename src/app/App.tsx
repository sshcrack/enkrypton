import { useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { Button, Flex, FormControl, FormLabel, Input, Text } from '@chakra-ui/react';
import Hostname from './Hostname';
import ConnectForm from './ConnectForm';

function App() {
  const [isTor, setIsTor] = useState<boolean | null>(null);
  const [ connected, setConnected] = useState(false)

  const checkTor = () => {
    invoke("tor_check").then(e => {
      console.log("Status reply", e)
      setIsTor(e as boolean)
    })
  }
  return <Flex w='100%' h='100%'>
    <Flex h='100%' w='30%'>
      
    </Flex>
    <Flex flex='1' h='100%' w='70%'>

    </Flex>
  </Flex>
}

export default App;

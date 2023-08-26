import { useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { Button, Flex, Text } from '@chakra-ui/react';

function App() {
  const [isTor, setIsTor] = useState<boolean | null>(null);

  const checkTor = () => {
    invoke("tor_check").then(e => {
      console.log("Status reply", e)
      setIsTor(e as boolean)
    })
  }
  return (
    <Flex
      w='100%'
      h='100%'
      gap='10'
      justifyContent='center'
      alignItems='center'
      flexDir='column'
    >
      <Text>{isTor === null ? "not checked yet" : (isTor ? "Is tor" : "no tor conn")}</Text>
      <Button onClick={checkTor} colorScheme='blue'>Check</Button>
    </Flex>
  );
}

export default App;

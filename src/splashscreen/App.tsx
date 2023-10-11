import { useEffect, useState } from 'react'
import storage from "../bindings/storage"
import { Button, Flex, Input, Text, useToast } from '@chakra-ui/react'
import TorStart from './TorStart';

export default function App() {
  const toast = useToast();

  const [unlocked, setUnlocked] = useState(null as boolean | null)
  const [pwd, setPwd] = useState("");
  const [loading, setLoading] = useState(false)


  useEffect(() => {
    storage.is_unlocked()
      .then(unlocked => setUnlocked(unlocked))
  }, [])

  const unlock = () => {
    setLoading(true)
    storage.unlockOrCreate(pwd)
      .then(() => {
        setUnlocked(true)
      })
      .catch(e => toast({
        title: "Error",
        description: e
      }))
      .finally(() => setLoading(false))
  }

  if (!unlocked) {
    return <Flex w='100%' h='100%' flexDir='column' justifyContent='space-between'>
      <Text>Unlocked: {unlocked ? "true" : "false"}</Text>

      <Input type="text" placeholder='Enter password' value={pwd} onChange={e => setPwd(e.target.value)} />
      <Button isLoading={loading} colorScheme='green' onClick={() => unlock()}>Submit</Button>
    </Flex>
  }


  return <TorStart />
}
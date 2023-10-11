import { useEffect, useState } from 'react'
import storage from "../bindings/storage"
import {
  Button,
  Flex,
  FormControl,
  FormErrorMessage,
  FormHelperText,
  FormLabel, Heading,
  Input,
  useToast
} from '@chakra-ui/react'
import TorStart from './TorStart';

export default function App() {
  const toast = useToast();

  const [unlocked, setUnlocked] = useState(null as boolean | null)
  const [pwd, setPwd] = useState("");
  const [loading, setLoading] = useState(false)
  const [passInvalid, setPassInvalid] = useState(false);

  useEffect(() => {
    storage.is_unlocked()
      .then(unlocked => setUnlocked(unlocked))
  }, [])

  const unlock = () => {
    setLoading(true)
    storage.unlockOrCreate(pwd)
      .then(() => {
        setUnlocked(true)
        setPassInvalid(false)
      })
      .catch(e => {
        if (e.toLowerCase().includes("invalid password")) {
          setPassInvalid(true)
          return
        }

        toast({
          title: "Error",
          description: e
        })
      })
      .finally(() => setLoading(false))
  }

  if (!unlocked) {
    return <Flex w='100%' h='100%' flexDir='column' justifyContent='space-between' p='2'>
      <Heading size='md'>Unlock Enkrypton</Heading>

      <FormControl isInvalid={passInvalid || pwd.length === 0}>
        <FormLabel>Password</FormLabel>
        <Input
          type='password'
          value={pwd}
          onChange={e => setPwd(e.target.value)}
          autoFocus
          onKeyUp={(e) => e.key === "Enter" && !loading && unlock()}
        />
        {passInvalid ? (
          <FormErrorMessage>
            Password is invalid.
          </FormErrorMessage>
        ) :
          pwd.length === 0 ? (
            <FormErrorMessage>
              Password is too short
            </FormErrorMessage>
          ) :
            (
              <FormHelperText>Enter password to enter.</FormHelperText>
            )}
      </FormControl>
      <Button isLoading={loading} colorScheme='green' onClick={() => unlock()} w='50%' alignSelf='center'>Submit</Button>
    </Flex>
  }


  return <TorStart />
}
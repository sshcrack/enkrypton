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

/**
 * Creates the splashcreen for the app (which includes the unlock screen of the storage).
 */
export default function App() {
  const toast = useToast();

  const [unlocked, setUnlocked] = useState(null as boolean | null)
  const [pwd, setPwd] = useState("");
  const [loading, setLoading] = useState(false)
  const [passInvalid, setPassInvalid] = useState(false);
  const [submittedOnce, setSubmittedOnce] = useState(false);

  // Maybe the storage is already unlocked? Checking...
  useEffect(() => {
    storage.is_unlocked()
      .then(unlocked => setUnlocked(unlocked))
  }, [])

  // Trying to unlock with given password
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

        // Telling the user which error occurred and logging it to console
        console.error(e)
        toast({
          title: "Error",
          description: e,
          status: "error"
        })
      })
      .finally(() => {
        setLoading(false)
        setSubmittedOnce(true)
      })
  }

  // Showing unlock screen if storage is locked
  if (!unlocked) {
    return <Flex w='100%' h='100%' flexDir='column' justifyContent='space-between' p='2'>
      <Heading size='md'>Unlock Enkrypton</Heading>

      <FormControl isInvalid={(passInvalid || pwd.length === 0) && submittedOnce}>
        <FormLabel>Password</FormLabel>
        <Input
          type='password'
          value={pwd}
          onChange={e => setPwd(e.target.value)}
          autoFocus
          onKeyUp={(e) => e.key === "Enter" && !loading && unlock()}
        />
        {passInvalid ?
        (
          <FormErrorMessage>
            Password is invalid.
          </FormErrorMessage>
        ) :
        (
          pwd.length === 0 ?
            <FormErrorMessage>Password is too short</FormErrorMessage> :
            <FormHelperText>Enter password to unlock.</FormHelperText>
        )}
      </FormControl>
      <Button isLoading={loading} colorScheme='green' onClick={() => unlock()} w='50%' alignSelf='center'>Submit</Button>
    </Flex>
  }


  return <TorStart />
}
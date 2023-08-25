import { Button, Text } from '@fluentui/react-components';
import "./App.scss";
import { useState } from 'react';
import { invoke } from '@tauri-apps/api';

function App() {
  const [isTor, setIsTor] = useState<boolean | null>(null);

  const checkTor = () => {
    invoke("tor_check").then(e => {
      console.log("Status reply", e)
      setIsTor(e as boolean)
    })
  }
  return (
    <div className="full-size flex-center flex flex-column">
      <Text>{isTor === null ? "not checked yet" : (isTor ? "Is tor" : "no tor conn")}</Text>
      <Button onClick={checkTor}>Check</Button>
    </div>
  );
}

export default App;

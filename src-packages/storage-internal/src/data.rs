use std::collections::HashMap;

use encryption::{PublicKey, PrivateKey, generate_pair};
use serde::{Deserialize, Serialize};

use ts_rs::TS;
use zeroize::{Zeroize, ZeroizeOnDrop};


// Only one Storage instance is allowed.
#[derive(TS, Clone, Debug, Zeroize, ZeroizeOnDrop, Deserialize, Serialize)]
#[ts(export)]
pub struct StorageData {
    // don't just skip this
    #[zeroize(skip)]
    pub nicknames: HashMap<String, String>,
    //REVIEW don't just skip this, key is pretty important to erase
    #[zeroize(skip)]
    pub chats: HashMap<String, StorageChat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop, TS)]
#[ts(export)]
pub struct StorageChat {
    #[zeroize(skip)]
    pub messages: Vec<ChatMessage>,
    pub nickname: Option<String>,

    // Remote public key
    #[ts(skip)]
    #[zeroize(skip)]
    pub pub_key: Option<PublicKey>,
    #[zeroize(skip)]
    pub receiver_onion: String,
    // Private Key of this client (to decrypt messages)
    #[zeroize(skip)]
    #[ts(skip)]
    pub priv_key: PrivateKey,
}

impl StorageChat {
    pub fn new(receiver_onion: &str) -> Self {
        Self {
            receiver_onion: receiver_onion.to_string(),
            messages: Vec::new(),
            nickname: None,

            pub_key: None,
            priv_key: generate_pair()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub self_sent: bool,
    //NOTE This message should not be lying around in memory unencrypted I guess
    pub msg: String,
    pub date: u128,
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            nicknames: HashMap::new(),
            chats: HashMap::new(),
        }
    }
}

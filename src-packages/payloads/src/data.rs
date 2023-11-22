use std::collections::HashMap;

use encryption::{PublicKey, PrivateKey, generate_pair};
use serde::{Deserialize, Serialize};

use zeroize::{Zeroize, ZeroizeOnDrop};
#[cfg(feature="export_ts")]
use ts_rs::TS;


// Only one Storage instance is allowed.
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop, Deserialize, Serialize)]
pub struct StorageData {
    // don't just skip this
    #[zeroize(skip)]
    pub nicknames: HashMap<String, String>,
    //REVIEW don't just skip this, key is pretty important to erase
    #[zeroize(skip)]
    pub chats: HashMap<String, StorageChat>,
}

#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct StorageChat {
    #[zeroize(skip)]
    pub messages: Vec<ChatMessage>,
    pub nickname: Option<String>,

    // Remote public key
    #[cfg_attr(feature="export_ts", ts(skip))]
    #[zeroize(skip)]
    pub rec_pub_key: Option<PublicKey>,
    #[zeroize(skip)]
    pub receiver_onion: String,
    // Private Key of this client (to decrypt messages)
    #[zeroize(skip)]
    #[cfg_attr(feature="export_ts", ts(skip))]
    pub priv_key: PrivateKey,
}

impl StorageChat {
    pub fn new(receiver_onion: &str) -> Self {
        Self {
            receiver_onion: receiver_onion.to_string(),
            messages: Vec::new(),
            nickname: None,

            rec_pub_key: None,
            priv_key: generate_pair()
        }
    }
}

#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct ChatMessage {
    pub self_sent: bool,
    //NOTE This message should not be lying around in memory unencrypted I guess
    pub msg: String,
    #[cfg_attr(feature="export_ts", ts(type="number"))]
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

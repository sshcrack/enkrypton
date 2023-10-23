use std::collections::HashMap;

use openssl::{
    pkey::{Private, Public},
    rsa::Rsa,
};
use serde::{Deserialize, Serialize, ser::SerializeStruct};

use ts_rs::TS;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::encryption::{generate_pair, PublicKey, PrivateKey};

// Only one Storage instance is allowed.
#[derive(TS, Clone, Debug, Zeroize, ZeroizeOnDrop, Deserialize, Serialize)]
#[ts(export)]
pub struct StorageData {
    //FIXME don't just skip this
    #[zeroize(skip)]
    pub nicknames: HashMap<String, String>,
    //FIXME don't just skip this, key is pretty important to erase
    #[zeroize(skip)]
    pub chats: HashMap<String, StorageChat>,
    #[ts(skip)]
    //FIXME don't just skip this, key is pretty important to erase
    #[zeroize(skip)]
    pub priv_key: PrivateKey,
}

#[derive(Clone, Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop, TS)]
#[ts(export)]
pub struct StorageChat {
    #[zeroize(skip)]
    pub messages: Vec<ChatMessage>,
    pub nickname: Option<String>,
    #[ts(skip)]
    #[zeroize(skip)]
    pub pub_key: PublicKey,
    #[zeroize(skip)]
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub self_sent: bool,
    //TODO Add actual encryption
    pub msg: String,
    pub date: usize,
}

impl Default for StorageData {
    fn default() -> Self {
        let key = generate_pair();

        Self {
            nicknames: HashMap::new(),
            chats: HashMap::new(),
            priv_key: key
        }
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use ts_rs::TS;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Only one Storage instance is allowed.
#[derive(TS, Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
#[ts(export)]
pub struct StorageData {
    //FIXME don't just skip this
    #[zeroize(skip)]
    pub nicknames: HashMap<String, String>,
    //FIXME don't just skip this, key is pretty important to erase
    #[zeroize(skip)]
    pub chats: HashMap<String, StorageChat>,
}


#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop, TS)]
#[ts(export)]
pub struct StorageChat {
    #[zeroize(skip)]
    pub messages: Vec<ChatMessage>,
    #[zeroize(skip)]
    pub nickname: Option<String>,
    #[zeroize(skip)]
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub self_sent: bool,
    //TODO Add actual encryption
    pub msg: String,
    pub date: usize
}


impl Default for StorageData {
    fn default() -> Self {
        Self {
            nicknames: HashMap::new(),
            chats: HashMap::new(),
        }
    }
}
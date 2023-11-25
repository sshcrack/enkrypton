use std::collections::HashMap;

use encryption::{PublicKey, PrivateKey, generate_pair};
use serde::{Deserialize, Serialize};

use zeroize::{Zeroize, ZeroizeOnDrop};
#[cfg(feature="export_ts")]
use ts_rs::TS;

use crate::payloads::WsMessageStatus;


// Only one Storage instance is allowed.
/// This struct includes all data that can be stored on the disk
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop, Deserialize, Serialize)]
pub struct StorageData {
    //REVIEW don't just skip this, key is pretty important to erase
    #[zeroize(skip)]
    /// All chats that are stored on the disk stored by the onion address of the receiver and the chat
    pub chats: HashMap<String, StorageChat>,
}

/// This chat contains all information about a receiver such as messages and keypairs
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct StorageChat {
    /// All messages sent to this receiver or received from this receiver
    #[zeroize(skip)]
    pub messages: Vec<ChatMessage>,
    /// The nickname of the receiver
    pub nickname: Option<String>,

    /// The public key which is used to encrypt messages when being sent to the receiver
    #[cfg_attr(feature="export_ts", ts(skip))]
    #[zeroize(skip)]
    pub rec_pub_key: Option<PublicKey>,
    #[zeroize(skip)]
    pub receiver_onion: String,
    /// Private key of ourselves used to decrypt the messages that are being received
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

/// A message that contains the message itself, the date, if it was sent by ourselves and the status of this message
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct ChatMessage {
    pub self_sent: bool,
    #[zeroize(skip)]
    pub status: WsMessageStatus,
    //NOTE This message should not be lying around in memory unencrypted I guess
    pub msg: String,
    #[cfg_attr(feature="export_ts", ts(type="number"))]
    pub date: u128,
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            chats: HashMap::new(),
        }
    }
}

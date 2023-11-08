use encryption::PublicKey;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub hostname: String,
    pub signature: Vec<u8>,
    pub pub_key: PublicKey,
}
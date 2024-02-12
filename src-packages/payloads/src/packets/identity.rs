use encryption::PublicKey;
use serde::{Serialize, Deserialize};


/// The identity of a client or server used to well verify the identity of the given side
#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    /// The hostname of the client / server
    pub hostname: String,
    /// The signature of that hostname, used to verify its identity (derived from the generated RSA Private Key)
    pub signature: Vec<u8>,
    /// And the public key that should be used when sending messages to the side
    pub pub_key: PublicKey,
}
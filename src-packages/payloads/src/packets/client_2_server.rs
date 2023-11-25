use serde::{Deserialize, Serialize};
use super::Identity;


/// All possible packets that can be sent from the client to the server
#[derive(Debug, Serialize, Deserialize)]
pub enum C2SPacket {
    /// Sends over the identity of this client and the public key / hostname with it.
    /// Contains a signature to verify the identity.
    SetIdentity(Identity),
    /// A packet to tell the client that their identity has been verified
    IdentityVerified,
    /// A new message from the client to the server
    Message((u128, Vec<u8>)),
    /// Tell the server that the message with the given date could be successfully received
    MessageReceived(u128),
    /// And again, tell the server that the message was failed to send
    MessageFailed(u128)
}
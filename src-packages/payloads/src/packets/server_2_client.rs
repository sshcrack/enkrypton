use serde::{Deserialize, Serialize};
use super::Identity;

/// All possible packets that can be sent from the server to the client
#[derive(Debug, Serialize, Deserialize)]
pub enum S2CPacket {
    /// A packet to verify the server on client side. Again, contains the identity struct
    VerifyIdentity(Identity),
    /// Used to tell the client that its identity has been verified successfully
    IdentityVerified,
    /// A message from the server to the client
    Message((u128, Vec<u8>)),
    /// Tell the client that the message with the given date could be successfully received
    MessageReceived(u128),
    /// And again, tell the client that the message was failed to send
    MessageFailed(u128)
}
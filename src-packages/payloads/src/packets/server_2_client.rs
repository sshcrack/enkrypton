use serde::{Deserialize, Serialize};
use super::Identity;

#[derive(Debug, Serialize, Deserialize)]
pub enum S2CPacket {
    DisconnectMultipleConnections,
    VerifyIdentity(Identity),
    IdentityVerified,
    Message((u128, Vec<u8>)),
    MessageReceived(u128),
    MessageFailed(u128)
}
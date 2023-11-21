use serde::{Deserialize, Serialize};
use super::Identity;

#[derive(Debug, Serialize, Deserialize)]
pub enum S2CPacket {
    DisconnectMultipleConnections,
    VerifyIdentity(Identity),
    IdentityVerified,
    Message(Vec<u8>),
}
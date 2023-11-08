use serde::{Deserialize, Serialize};
use super::Identity;

#[derive(Serialize, Deserialize)]
pub enum S2CPacket {
    DisconnectMultipleConnections,
    VerifyIdentity(Identity),
    IdentityVerified,
    Message(Vec<u8>),
}
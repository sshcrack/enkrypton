use serde::{Deserialize, Serialize};
use super::Identity;


#[derive(Debug, Serialize, Deserialize)]
pub enum C2SPacket {
    SetIdentity(Identity),
    IdentityVerified,
    Message(Vec<u8>),
}
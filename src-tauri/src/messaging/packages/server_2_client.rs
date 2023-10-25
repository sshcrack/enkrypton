use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub enum S2CPacket {
    DisconnectMultipleConnections,
    IdentityVerified
}
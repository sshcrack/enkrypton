use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum WsClientStatus {
    CONNECTED,
    DISCONNECTED,
}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WsClientUpdate {
    pub hostname: String,
    pub status: WsClientStatus
}
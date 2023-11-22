use serde::{Serialize, Deserialize};

use crate::event::Sendable;
#[cfg(feature="export_ts")]
use ts_rs::TS;

#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsClientStatus {
    ConnectingProxy,
    ConnectingHost,
    Connected,
    WaitingIdentity,
    Done,
    Disconnected
}


#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsClientUpdatePayload {
    pub hostname: String,
    pub status: WsClientStatus
}

impl Sendable for WsClientUpdatePayload {
    fn get_name(&self) -> String {
        "ws_client_update".to_string()
    }
}
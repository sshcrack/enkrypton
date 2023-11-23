use serde::{Serialize, Deserialize};

use crate::event::Sendable;
#[cfg(feature="export_ts")]
use ts_rs::TS;

#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessageStatus {
    Failed,
    Success,
    Sending,
    Sent
}


#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessageStatusPayload {
    pub hostname: String,
    pub status: WsMessageStatus
}

impl Sendable for WsMessageStatusPayload {
    fn get_name(&self) -> String {
        "ws_msg_update".to_string()
    }
}
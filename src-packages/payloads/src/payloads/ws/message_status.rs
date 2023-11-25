use serde::{Serialize, Deserialize};

use crate::event::Sendable;
#[cfg(feature="export_ts")]
use ts_rs::TS;

/// Different states of the message status (failed, success, sending, sent)
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessageStatus {
    Failed,
    Success,
    Sending,
    Sent
}


/// A payload to tell the frontend of the status of a message
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessageStatusPayload {
    pub hostname: String,
    pub status: WsMessageStatus,
    #[cfg_attr(feature="export_ts", ts(type="number"))]
    pub date: u128
}

impl Sendable for WsMessageStatusPayload {
    fn get_name(&self) -> String {
        "ws_msg_update".to_string()
    }
}
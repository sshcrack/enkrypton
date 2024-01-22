use serde::{Serialize, Deserialize};

use crate::event::Sendable;
#[cfg(feature="export_ts")]
use ts_rs::TS;

/// Different states of the message status (failed, success, sending, sent)
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessageStatus {
    /// That the message failed to send to the receiver
    Failed,
    /// The message was successfully sent to the receiver
    Success,
    /// The message is currently being sent to the receiver
    Sending,
    /// The message has been sent but not yet received/decrypted by the receiver
    Sent
}


/// A payload to tell the frontend of the status of a message
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessageStatusPayload {
    /// The name of the receiver
    pub hostname: String,
    /// The new status of the message
    pub status: WsMessageStatus,
    #[cfg_attr(feature="export_ts", ts(type="number"))]
    /// The date/id of the message
    pub date: u128
}

impl Sendable for WsMessageStatusPayload {
    fn get_name(&self) -> String {
        "ws_msg_update".to_string()
    }
}
use serde::{Serialize, Deserialize};

use crate::event::SendablePayload;
#[cfg(feature="export_ts")]
use ts_rs::TS;


/// Payload is used to send newly arrived messages to the frontend
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessagePayload {
    /// The hostname of the receiver that should receive this message
    pub receiver: String,
    /// The message that should be sent to the receiver
    pub message: String
}

impl SendablePayload for WsMessagePayload {
    fn get_name(&self) -> String {
        format!("msg-{}", self.receiver)
    }
}
use serde::{Serialize, Deserialize};

use crate::event::Sendable;
#[cfg(feature="export_ts")]
use ts_rs::TS;


/// Payload is used to send newly arrived messages to the frontend
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessagePayload {
    pub receiver: String,
    pub message: String
}

impl Sendable for WsMessagePayload {
    fn get_name(&self) -> String {
        format!("msg-{}", self.receiver)
    }
}
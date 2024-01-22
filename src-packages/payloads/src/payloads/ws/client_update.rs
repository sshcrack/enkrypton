use serde::{Serialize, Deserialize};

use crate::event::Sendable;
#[cfg(feature="export_ts")]
use ts_rs::TS;

/// This enum includes the status of the websocket connection. Can be used from the server side or client side.
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsClientStatus {
    /// The websocket is connecting to the tor proxy
    ConnectingProxy,
    /// The client is connected to the tor proxy and is now connecting to the onion host
    ConnectingHost,
    /// The client is waiting for the identity to be verified
    WaitingIdentity,
    /// The connection was successful and we can send messages
    Connected,
    /// The client has been disconnected
    Disconnected
}


/// The payload that is sent to the frontend when the status of the websocket connection has changed
#[cfg_attr(feature="export_ts", derive(TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsClientUpdatePayload {
    /// The receiver to update with this payload
    pub hostname: String,
    /// The new status of the websocket connection
    pub status: WsClientStatus
}

impl Sendable for WsClientUpdatePayload {
    fn get_name(&self) -> String {
        "ws_client_update".to_string()
    }
}
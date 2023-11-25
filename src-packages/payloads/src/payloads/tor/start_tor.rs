use serde::{Serialize, Deserialize};

use crate::event::Sendable;

/// Used to tell the client what the progress of the tor startup is
#[cfg_attr(feature="export_ts", derive(ts_rs::TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct StartTorPayload {
    /// The percentage of the progress
    pub progress: f32,
    // And the latest message tor has sent
    pub message: String,
}

impl Sendable for StartTorPayload {
    fn get_name(&self) -> String {
        "tor_start".to_string()
    }
}


/// Used to tell the client that the tor startup has failed
#[cfg_attr(feature="export_ts", derive(ts_rs::TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorStartupErrorPayload {
    /// The last message that tor emitted
    pub message: String,
    /// The exit code of the tor process
    pub error_code: Option<i32>,
    /// And a truncated list of logs  emitted by tor
    pub logs: Option<Vec<String>>
}

impl Sendable for TorStartupErrorPayload {
    fn get_name(&self) -> String {
        "tor_start_error".to_string()
    }
}
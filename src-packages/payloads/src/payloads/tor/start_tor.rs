use serde::{Serialize, Deserialize};

use crate::event::Sendable;

#[cfg_attr(feature="export_ts", derive(ts_rs::TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct StartTorPayload {
    pub progress: f32,
    pub message: String,
}

impl Sendable for StartTorPayload {
    fn get_name(&self) -> String {
        "tor_start".to_string()
    }
}


#[cfg_attr(feature="export_ts", derive(ts_rs::TS))]
#[cfg_attr(feature="export_ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorStartupErrorPayload {
    pub message: String,
    pub error_code: Option<i32>,
    pub logs: Option<Vec<String>>
}

impl Sendable for TorStartupErrorPayload {
    fn get_name(&self) -> String {
        "tor_start_error".to_string()
    }
}
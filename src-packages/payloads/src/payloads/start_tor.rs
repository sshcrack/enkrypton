use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct StartTorPayload {
    pub progress: f32,
    pub message: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TorStartupErrorPayload {
    pub message: String,
    pub error_code: Option<i32>,
    pub logs: Option<Vec<String>>
}
use serde::{Serialize, Deserialize};
use ts_rs::TS;


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WsMessagePayload {
    pub message: String
}
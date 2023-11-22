use serde::{Deserialize, Serialize};

use crate::event::Sendable;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplashscreenClosedPayload {
}

impl Sendable for SplashscreenClosedPayload {
    fn get_name(&self) -> String {
        return "splashscreen_closed".to_string()
    }
}
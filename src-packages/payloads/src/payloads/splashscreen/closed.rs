use serde::{Deserialize, Serialize};

use crate::event::Sendable;


/// Just a payload to tell the client that the splashscreen has been closed.
/// The frontend will start to boot up after this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplashscreenClosedPayload {
}

impl Sendable for SplashscreenClosedPayload {
    fn get_name(&self) -> String {
        return "splashscreen_closed".to_string()
    }
}
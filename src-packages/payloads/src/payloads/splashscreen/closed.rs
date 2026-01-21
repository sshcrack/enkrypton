use serde::{Deserialize, Serialize};

use crate::event::SendablePayload;


/// Just a payload to tell the client that the splashscreen has been closed.
/// The frontend will start to boot up after this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplashscreenClosedPayload {
}

impl SendablePayload for SplashscreenClosedPayload {
    fn get_name(&self) -> String {
        "splashscreen_closed".to_string()
    }
}
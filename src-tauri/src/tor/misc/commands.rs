use log::{debug, warn};
use tauri::{Manager, Window};

use crate::tor::manager;

#[tauri::command()]
pub async fn tor_start(window: Window) -> Result<(), String> {
    let res = manager::start_tor(
        move |start_payload| {
            let res = window.app_handle().emit_all("tor_start", start_payload);
            if res.is_ok() {
                return;
            }

            warn!("Tor start could not send payload {:?}", res.unwrap_err())
        },
    )
    .await;

    if res.is_ok() {
        debug!("done command");
        return Ok(());
    }

    return Err(res.unwrap_err().to_string());
}

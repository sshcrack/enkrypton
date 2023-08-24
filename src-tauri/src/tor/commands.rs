use log::{warn, debug};
use tauri::{Manager, Window};

use crate::tor::manager;

#[tauri::command()]
pub async fn tor_start(window: Window) -> Result<(), String> {
    let res = manager::start_tor(move |e| {
        println!("tor_start: New event: {} ({}%)", e.message, e.progress * 100.0);
        let res = window.app_handle().emit_all("tor_start", e);
        if res.is_ok() { return; }

        warn!("Tor start could not send payload {:?}", res.unwrap_err())
    }).await;

    if res.is_ok() {
        debug!("done command");
        return Ok(());
    }

    return Err(res.unwrap_err().to_string());
}

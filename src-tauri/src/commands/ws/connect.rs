use log::debug;
use messaging::general::MESSAGING;

use crate::util::is_onion_hostname;

/// Connects to the given onion_hostname and returns if there is already a connection
#[tauri::command]
pub async fn ws_connect(onion_hostname: String) -> Result<(), String> {
    if !is_onion_hostname(&onion_hostname) {
        let should_skip = cfg!(feature="dev") && (onion_hostname.ends_with("-dev-client") || onion_hostname.ends_with("-dev-server"));

        if !should_skip {
            return Err("Invalid onion hostname".to_string());
        }
    }

    debug!("Getting or creating client...");
    if MESSAGING.read().await.is_connected(&onion_hostname).await {
        return Ok(());
    }

    MESSAGING
        .write()
        .await
        .get_or_connect(&onion_hostname)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

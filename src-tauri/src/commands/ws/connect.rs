use log::debug;

use crate::messaging::MESSAGING;

#[tauri::command]
pub async fn ws_connect(onion_hostname: String) -> Result<(), String> {
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

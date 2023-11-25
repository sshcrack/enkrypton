use log::debug;
use messaging::general::MESSAGING;


/// Sends a message to the receiver with the message
#[tauri::command]
pub async fn ws_send(onion_hostname: String, msg: String) -> Result<(), String> {
    debug!("Sending {} to {}", msg, onion_hostname);

    let manager = MESSAGING.read().await;
    debug!("Getting...");
    let conn = manager.get_or_connect(&onion_hostname).await
        .map_err(|e| e.to_string())?;

    debug!("Waiting until verified...");

    manager.wait_until_verified(&onion_hostname).await.map_err(|e| e.to_string())?;
    debug!("Sending...");
    conn.send_msg(&msg).await
        .map_err(|e| e.to_string())?;

    debug!("Sent.");
    Ok(())
}

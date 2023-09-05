use log::{debug, error};

use crate::client::{ClientManager, MessagingClient};

#[tauri::command]
pub async fn ws_send(onion_hostname: String, msg: String) -> Result<(), String> {
    debug!("Sending {} to {}", msg, onion_hostname);

    let client = MessagingClient::get_or_create(&onion_hostname)
        .await
        .or_else(|e| {
            error!("{}", e);
            Err(e.to_string())
        })?;

    client.send_msg_txt(&msg).await
        .or_else(|e| {
            error!("{}", e);
            Err(e.to_string())
        })?;

    Ok(())
}

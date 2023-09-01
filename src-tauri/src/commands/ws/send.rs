use log::{debug, error};

use crate::client::{ClientManager, MessagingClient};

#[tauri::command]
pub async fn ws_send(onion_addr: String, msg: String) -> Result<(), String> {
    debug!("Sending {} to {}", msg, onion_addr);

    let client = MessagingClient::get_or_create(&onion_addr)
        .await
        .or_else(|e| {
            error!("{}", e);
            Err(e.to_string())
        })?;

    client.send_msg(&msg).await
        .or_else(|e| {
            error!("{}", e);
            Err(e.to_string())
        })?;

    Ok(())
}

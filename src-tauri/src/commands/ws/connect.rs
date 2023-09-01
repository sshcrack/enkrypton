use log::{debug, error};

use crate::client::{ClientManager, MessagingClient};

#[tauri::command]
pub async fn ws_connect(onion_addr: String) -> Result<(), String> {
    debug!("Getting or creating client...");

    MessagingClient::get_or_create(&onion_addr)
        .await
        .or_else(|e| {
            error!("{}", e);
            Err(e.to_string())
        })?;

    Ok(())
}

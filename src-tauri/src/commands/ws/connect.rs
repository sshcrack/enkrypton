use log::{debug, error};

use crate::client::{ClientManager, MessagingClient};

#[tauri::command]
pub async fn ws_connect(onion_hostname: String) -> Result<(), String> {
    debug!("Getting or creating client...");

    MessagingClient::get_or_create(&onion_hostname)
        .await
        .or_else(|e| {
            error!("{}", e);
            Err(e.to_string())
        })?;

    Ok(())
}

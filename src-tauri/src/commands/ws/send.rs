use log::{debug, error};


#[tauri::command]
pub async fn ws_send(onion_hostname: String, msg: String) -> Result<(), String> {
    debug!("Sending {} to {}", msg, onion_hostname);

    //TODO [...]
    Ok(())
}

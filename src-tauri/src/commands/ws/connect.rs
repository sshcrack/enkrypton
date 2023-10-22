use log::{debug, error};

#[tauri::command]
pub async fn ws_connect(onion_hostname: String) -> Result<(), String> {
    debug!("Getting or creating client...");

    //[...]
    Ok(())
}

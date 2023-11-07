use log::debug;

use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_exists() -> Result<bool, String> {
    debug!("Reading storage...");
    Ok(STORAGE.read().await.exists())
}

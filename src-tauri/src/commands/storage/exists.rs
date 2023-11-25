use log::debug;

use storage_internal::STORAGE;

// Checks if the storage file even exists
#[tauri::command]
pub async fn storage_exists() -> Result<bool, String> {
    debug!("Reading storage...");
    Ok(STORAGE.read().await.exists())
}

use log::debug;

use storage_internal::STORAGE;

/// Returns wether the storage is unlocked
#[tauri::command]
pub async fn storage_is_unlocked() -> Result<bool, String> {
    debug!("Check unlock...");
    Ok(STORAGE.read().await.is_unlocked())
}

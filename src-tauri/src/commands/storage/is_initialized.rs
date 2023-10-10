use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_is_initialized() -> Result<bool, String> {
    Ok(STORAGE.read().await.is_initialized())
}

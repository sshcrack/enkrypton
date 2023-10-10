use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_is_valid() -> Result<bool, String> {
    Ok(STORAGE.read().await.is_valid())
}

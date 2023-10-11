use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_exists() -> Result<bool, String> {
    Ok(STORAGE.read().await.exists())
}

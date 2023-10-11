use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_is_unlocked() -> Result<bool, String> {
    Ok(STORAGE.read().await.is_unlocked())
}

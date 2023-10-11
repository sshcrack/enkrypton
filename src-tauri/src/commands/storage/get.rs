use crate::storage::{StorageData, STORAGE};

#[tauri::command]
pub async fn storage_get() -> Result<StorageData, String> {
    let storage = STORAGE.write().await;
    if !storage.is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    let data = storage.data().await;
    if let Some(data) = data {
        return Ok(data);
    }

    Err("Storage data is not set.".to_string())
}

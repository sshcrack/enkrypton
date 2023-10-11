use crate::storage::{STORAGE};

#[tauri::command]
pub async fn storage_save() -> Result<(), String> {
    let mut storage = STORAGE.write().await;
    if !storage.is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    storage.save().await.or_else(|e| Err(e.to_string()))
}

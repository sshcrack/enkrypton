use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_delete() -> Result<(), String> {
    let mut storage = STORAGE.write().await;
    if !storage.is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    let res = storage.delete().await;
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }
    Ok(())
}

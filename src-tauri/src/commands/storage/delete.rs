use storage_internal::STORAGE;

#[tauri::command]
pub async fn storage_delete() -> Result<(), String> {
    if !(STORAGE.read().await).is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    let res = STORAGE.write().await.delete().await;
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }
    Ok(())
}

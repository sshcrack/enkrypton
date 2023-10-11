use crate::storage::{STORAGE, StorageData};

#[tauri::command]
pub async fn storage_set(data_raw: String) -> Result<(), String> {
    let mut storage = STORAGE.write().await;
    if !storage.is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    let data = serde_json::from_str::<StorageData>(&data_raw)
        .or_else(|e| Err(format!("Could not parse storage data: {}", e)))?;

    storage.modify_storage(|e| e.data = Some(data)).await?;

    Ok(())
}

use payloads::data::StorageData;
use storage_internal::STORAGE;

#[tauri::command]
pub async fn storage_get() -> Result<StorageData, String> {
    if !(STORAGE.read().await).is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    let data = STORAGE.read().await.data().await;
    if let Some(data) = data {
        return Ok(data);
    }

    Err("Storage data is not set.".to_string())
}

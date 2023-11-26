use payloads::data::StorageData;
use storage_internal::STORAGE;
use crate::util::assert_unlocked_str;

/// Gets the storage data if the storage is unlocked
#[tauri::command]
pub async fn storage_get() -> Result<StorageData, String> {
    assert_unlocked_str().await?;

    let data = STORAGE.read().await.data().await;
    if let Some(data) = data {
        return Ok(data);
    }

    Err("Storage data is not set.".to_string())
}

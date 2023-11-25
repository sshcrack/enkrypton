use log::debug;
use payloads::data::StorageData;
use storage_internal::STORAGE;

/// Sets the data of the storage if the storage is unlocked
#[tauri::command]
pub async fn storage_set(data_raw: String) -> Result<(), String> {
    if !(STORAGE.read().await).is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    // Parsing the data
    let data = serde_json::from_str::<StorageData>(&data_raw)
        .map_err(|e| format!("Could not parse storage data: {}", e))?;

    // And modifying the storage file
    STORAGE.read().await
        .modify_storage_data(move |e| {
            *e = data;

            Ok(())
        })
        .await
        .map_err(|e| format!("Could not update storage: {:?}", e))?;

        debug!("Done");
    Ok(())
}

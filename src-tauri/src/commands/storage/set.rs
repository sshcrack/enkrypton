use log::debug;

use crate::storage::{StorageData, STORAGE};

#[tauri::command]
pub async fn storage_set(data_raw: String) -> Result<(), String> {
    if !(STORAGE.read().await).is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    let data = serde_json::from_str::<StorageData>(&data_raw)
        .map_err(|e| format!("Could not parse storage data: {}", e))?;

    debug!("Modify storage");
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

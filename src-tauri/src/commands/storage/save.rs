use log::debug;

use storage_internal::STORAGE;
use crate::util::assert_unlocked_str;

/// Saves the storage if the storage is unlocked
#[tauri::command]
pub async fn storage_save() -> Result<(), String> {
    assert_unlocked_str().await?;

    debug!("Saving storage...");
    let r = STORAGE.read().await.save().await.or_else(|e| Err(e.to_string()));
    debug!("Done");

    r
}

use log::debug;

use storage_internal::STORAGE;

#[tauri::command]
pub async fn storage_save() -> Result<(), String> {
    if !(STORAGE.read().await).is_unlocked() {
        return Err("Storage is not unlocked".to_string());
    }

    debug!("Saving storage...");
    let r = STORAGE.read().await.save().await.or_else(|e| Err(e.to_string()));
    debug!("Done");

    r
}

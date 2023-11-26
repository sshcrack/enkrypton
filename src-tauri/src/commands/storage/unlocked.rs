use log::debug;

use crate::util::assert_unlocked_str;

/// Returns wether the storage is unlocked
#[tauri::command]
pub async fn storage_is_unlocked() -> Result<bool, String> {
    debug!("Check unlock...");
    Ok(assert_unlocked_str().await.is_ok())
}

use storage_internal::STORAGE;
use crate::util::assert_unlocked_str;

/// Deletes the current storage if the user does not remember the password
#[tauri::command]
pub async fn storage_delete() -> Result<(), String> {
    assert_unlocked_str().await?;

    let res = STORAGE.write().await.delete().await;
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }
    Ok(())
}

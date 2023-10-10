use log::error;

use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_initialize(pass: &str) -> Result<(), String> {
    //TODO beautify
    let res = STORAGE.write().await
    .initialize(pass).await;

    if res.is_err() {
        let e = res.unwrap_err();
        error!("Error initializing storage: {}", e);

        return Err(e.to_string());
    }

    return Ok(());
}

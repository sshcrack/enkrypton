use anyhow::Result;
use log::error;

use crate::storage::STORAGE;

#[tauri::command]
pub async fn storage_unlock_or_create(pass: &str) -> Result<(), String> {
    let res = inner_func(pass).await;

    if res.is_err() {
        let e = res.unwrap_err();
        error!("Could not unlock storage: {}", e);

        return Err(e.to_string());
    }

    Ok(())
}

pub async fn inner_func(pass: &str) -> Result<()> {
    let mut state = STORAGE.write().await;
    if !state.has_parsed() {
        state.read_or_generate(pass).await?;

        if state.is_unlocked() {
            return Ok(());
        }
    }

    state.try_unlock(pass.as_bytes())?;
    return Ok(());
}

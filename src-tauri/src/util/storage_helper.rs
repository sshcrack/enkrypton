use storage_internal::STORAGE;

/// Returns with a string error if the storage has not been unlocked yet
pub async fn assert_unlocked_str() -> Result<(), String> {
    let st = STORAGE.read().await;
    let unlocked = st.is_unlocked().or_else(|e| Err(e.to_string()))?;
    if !unlocked {
        return Err("This command requires the storage to be unlocked.".to_string());
    }

    Ok(())
}
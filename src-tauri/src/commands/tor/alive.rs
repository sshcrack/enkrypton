use crate::tor::{config::TOR_CLIENT, consts::DEFAULT_HTTP_RETURN};

#[tauri::command]
pub async fn tor_is_alive(_addr: String) -> Result<bool, String> {
    let res = TOR_CLIENT
        .get("/")
        .send()
        .await
        .or_else(|e| Err(e.to_string()))?;

    let text = res.text().await.or_else(|e| Err(e.to_string()))?;

    Ok(text == *DEFAULT_HTTP_RETURN)
}

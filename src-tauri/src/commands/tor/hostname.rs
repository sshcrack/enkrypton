use tor_proxy::{service::get_service_hostname, consts::TOR_START_LOCK};

use crate::util::to_str_err;

/// Gets the hostname of the tor service
#[tauri::command]
pub async fn tor_hostname() -> Result<String, String> {
    // Wait for the tor start to finish first
    let _ = TOR_START_LOCK.read().await;
    let res = get_service_hostname(false).await.or_else(|e| to_str_err(e)())?;

    if let Some(res) = res {
        if res.is_empty() {
            return Err("Hostname is empty, tor has probably not started".to_string());
        }

        return Ok(res);
    }

    return Err("File is empty, tor has probably not started".to_string());
}

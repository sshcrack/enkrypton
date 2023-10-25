use crate::{
    tor::{consts::TOR_START_LOCK, service::get_service_hostname},
    util::to_str_err,
};

#[tauri::command]
pub async fn tor_hostname() -> Result<String, String> {
    let _ = TOR_START_LOCK.read().await;
    let res = get_service_hostname().await.or_else(|e| to_str_err(e)())?;

    if let Some(res) = res {
        if res.is_empty() {
            return Err("Hostname is empty, tor has probably not started".to_string());
        }

        return Ok(res);
    }

    return Err("File is empty, tor has probably not started".to_string());
}

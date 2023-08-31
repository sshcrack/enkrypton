use crate::{tor::service::get_service_hostname, util::to_str_err};

#[tauri::command]
pub async fn tor_hostname() -> Result<Option<String>, String> {
    let res = get_service_hostname().or_else(|e| to_str_err(e)())?;

    Ok(res)
}

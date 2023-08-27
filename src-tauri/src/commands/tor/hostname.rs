use crate::tor::service::get_service_hostname;

#[tauri::command]
pub async fn tor_hostname() -> Result<Option<String>, String> {
    let res = get_service_hostname();
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }

    Ok(res.unwrap())
}

use crate::{tor::config::TOR_CLIENT, util::to_str_err};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct TorCheckResponse {
    #[serde(rename = "IsTor")]
    is_tor: bool,
    #[serde(rename = "IP")]
    ip: String,
}

/* checks if the client is in the tor network */
#[tauri::command()]
pub async fn tor_check() -> Result<bool, String> {
    let mut res = TOR_CLIENT
        .get("https://check.torproject.org/api/ip")
        .send()
        .await
        .or_else(|e| to_str_err(e)())?;

    let status = res.status().await.or_else(|e| to_str_err(e)())?;
    let headers = res.headers().await.or_else(|e| to_str_err(e)())?;

    println!("Status: {}", status);
    println!("Headers:\n{:#?}", headers);

    let body = res
        .json::<TorCheckResponse>()
        .await
        .or_else(|e| to_str_err(e)())?;

    println!("Body:\n{:#?}", body);
    Ok(body.is_tor)
}

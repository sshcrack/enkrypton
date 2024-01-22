use log::debug;
use messaging::client::TOR_CLIENT;
use serde::Deserialize;

use crate::util::to_str_err;

#[derive(Deserialize, Debug, Clone)]
// Actually needed to parse the json
#[allow(dead_code)]
/// Represents the response from the tor check api
pub struct TorCheckResponse {
    #[serde(rename = "IsTor")]
    /// Whether the client is in the tor network
    is_tor: bool,
    #[serde(rename = "IP")]
    /// Our ip address of the exit node
    ip: String,
}

/// checks if the client is in the tor network
#[tauri::command()]
pub async fn tor_check() -> Result<bool, String> {
    let mut res = TOR_CLIENT
        .get("https://check.torproject.org/api/ip")
        .send()
        .await
        .or_else(|e| to_str_err(e)())?;

    let status = res.status().await.or_else(|e| to_str_err(e)())?;
    let headers = res.headers().await.or_else(|e| to_str_err(e)())?;

    debug!("Status: {}", status);
    debug!("Headers:\n{:#?}", headers);

    let body = res
        .json::<TorCheckResponse>()
        .await
        .or_else(|e| to_str_err(e)())?;

    debug!("Body:\n{:#?}", body);
    Ok(body.is_tor)
}

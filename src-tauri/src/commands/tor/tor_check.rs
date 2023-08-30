use crate::tor::config::TOR_CLIENT;

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
    let res = TOR_CLIENT
        .get("https://check.torproject.org/api/ip")
        .header("Accept", "*/*")
        .header("User-Agent", "Firefox")
        .send()
        .or_else(|e| Err(e.to_string()))?;

/*
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res.json::<TorCheckResponse>().await;
    if body.is_err() {
        return Err(body.unwrap_err().to_string());
    }
 */
    let body = res.text().unwrap();//body.unwrap();

    println!("Body:\n{:#?}", body);
    Ok(false)
    //Ok(body.is_tor)
}

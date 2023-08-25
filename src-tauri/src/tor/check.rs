use anyhow::Result;

use crate::tor::consts::TOR_CLIENT;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TorCheckResponse {
    IsTor: bool,
    IP: String,
}

/* checks if the client is in the tor network */
#[tauri::command]
pub async fn tor_check() -> Result<(), String> {
    let res = TOR_CLIENT
        .get("https://check.torproject.org/api/ip")
        .send()
        .await;
    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }

    let res 
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res.json::<TorCheckResponse>().await?;
    println!("Body:\n{:#?}", body);
    Ok(body.IsTor)
}

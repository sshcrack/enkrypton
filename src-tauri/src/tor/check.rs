use crate::tor::config::{TOR_CLIENT, CONFIG};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TorCheckResponse {
    IsTor: bool,
    IP: String,
}

/* checks if the client is in the tor network */
#[tauri::command()]
pub async fn tor_check() -> Result<bool, String> {
    let res = CONFIG.create_client().unwrap()
        .get("https://check.torproject.org/api/ip")
        .send()
        .await;

    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }

    let res = res.unwrap();
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res.json::<TorCheckResponse>().await;
    if body.is_err() {
        return Err(body.unwrap_err().to_string());
    }

    let body = body.unwrap();

    println!("Body:\n{:#?}", body);
    Ok(body.IsTor)
}

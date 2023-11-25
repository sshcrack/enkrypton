use anyhow::{anyhow, Result};
use log::debug;
use url::Url;

/// Returns the servername for the given url
pub fn get_servername(url: &Url) -> Result<String> {
    let host = url
        .host_str()
        .ok_or(anyhow!("Host is not in the url ({})", url))?;

    debug!("Parsing port");
    let port = url.port_or_known_default().unwrap_or(80);

    let formatted = format!("{}:{}", host, port);
    Ok(formatted)
}
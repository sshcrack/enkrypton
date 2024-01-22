use anyhow::{anyhow, Result};
use log::debug;
use url::Url;

/// Gets the server name from the url
///
/// # Arguments
///
/// * `url` - The url to get the server name from
///
/// # Returns
///
/// The server name
pub fn get_server_name(url: &Url) -> Result<String> {
    let host = url
        .host_str()
        .ok_or(anyhow!("Host is not in the url ({})", url))?;

    debug!("Parsing port");
    let port = url.port_or_known_default().unwrap_or(80);

    let formatted = format!("{}:{}", host, port);
    Ok(formatted)
}
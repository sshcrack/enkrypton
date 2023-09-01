use std::fmt::Display;

use anyhow::{anyhow, Result};
use log::{debug, error};
use url::Url;

pub fn to_str_err<E, K>(err: E) -> impl Fn() -> Result<K, String>
where
    E: ToString + Display,
{
    return move || {
        error!("Error: {}", err);
        Err(err.to_string())
    };
}

pub fn get_servername(url: &Url) -> Result<String> {
    let host = url
        .host_str()
        .ok_or(anyhow!("Host is not in the url ({})", url))?;

    debug!("Parsing port");
    let port = url
        .port_or_known_default()
        .unwrap_or(80);

    let formatted = format!("{}:{}", host, port);
    Ok(formatted)
}

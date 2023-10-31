use std::{fmt::Display, path::PathBuf, env::current_exe, fs::create_dir_all};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{debug, error};
use regex::Regex;
use tauri::AppHandle;
use url::Url;

use crate::{tor::{consts::APP_HANDLE, manager::{stop_tor, wait_for_exit}}, storage::STORAGE};


pub fn get_root_dir() -> PathBuf {
    let mut buf = current_exe().unwrap().parent().unwrap().to_path_buf();
    buf.push("enkrypton_root/");

    if !buf.is_dir() {
        create_dir_all(&buf).unwrap();
    }

    buf
}

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
    let port = url.port_or_known_default().unwrap_or(80);

    let formatted = format!("{}:{}", host, port);
    Ok(formatted)
}

pub async fn get_app() -> AppHandle {
    let state = APP_HANDLE.read().await;
    let handle = state.as_ref().unwrap();

    return handle.clone();
}

lazy_static! {
    pub static ref ONION_REGEX: Regex = Regex::new("^([A-z]|[0-9])+$").unwrap();
}

pub fn is_onion_hostname(addr: &str) -> bool {

    return ONION_REGEX.is_match(addr);
}





/// This function is called when the application is closed. It stops the tor process and saves the storage.
pub async fn on_exit() -> anyhow::Result<()> {
    debug!("Acquiring storage lock...");

    let mut e = STORAGE.write().await;
    debug!("Saving storage...");
    e.save().await?;

    stop_tor().await?;
    wait_for_exit().await;
    e.exit().await?;

    Ok(())
}

use std::{fmt::Display, path::PathBuf, env::current_exe, fs::create_dir_all, time::SystemTime};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{debug, error};
use regex::Regex;
use tauri::AppHandle;
use url::Url;

use crate::{tor::{consts::APP_HANDLE, manager::{stop_tor, wait_for_exit}}, storage::STORAGE};

pub fn to_str_err<E, K>(err: E) -> impl Fn() -> Result<K, String>
where
    E: ToString + Display,
{
    return move || {
        error!("Error: {}", err);
        Err(err.to_string())
    };
}

lazy_static! {
    pub static ref ONION_REGEX: Regex = Regex::new("^([A-z]|[0-9])+$").unwrap();
}

/// Used for validation of the given onion address
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
use std::fmt::Display;

use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, error};
use regex::Regex;
use storage_internal::STORAGE;
use tor_proxy::manager::{stop_tor, wait_for_exit};

/// A function to convert an error to a string.
/// Acts as a helper function for commands
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
    /// A regex used to verify the onion address
    pub static ref ONION_REGEX: Regex = Regex::new("^[a-z2-7]{56}$").unwrap();
}

/// Used for validation of the given onion address
pub fn is_onion_hostname(addr: &str) -> bool {
    return ONION_REGEX.is_match(addr);
}





/// This function is called when the application is closed. It stops the tor process and saves the storage.
pub async fn on_exit() -> Result<()> {
    debug!("Acquiring storage lock...");

    let mut e = STORAGE.write().await;
    debug!("Saving storage...");
    e.save().await?;

    stop_tor().await?;
    wait_for_exit().await;
    e.exit().await?;

    Ok(())
}
use std::fmt::Display;

use anyhow::Error;
use log::error;

pub fn to_str_err<E, K>(err: E) -> impl Fn() -> Result<K, String>
where
    E: ToString + Display,
{
    return move || {
        error!("Error: {}", err);
        Err(err.to_string())
    };
}

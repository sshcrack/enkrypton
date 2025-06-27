use std::{fs::File, io::{self, Cursor}};

use anyhow::Result;
use log::error;
use sha2::{Digest, Sha256};
use shared::get_root_dir;

use crate::consts::{TOR_BINARY_PATH, TOR_BINARY_HASH};

/// Checks for the binary of tor at TOR_BINARY_PATH and extracts the tor
/// binary if the file does not exist or the hash is wrong
pub fn check_integrity() -> Result<()> {
    let is_valid = is_tor_binary_valid().unwrap_or(false);

    if !is_valid {
        error!("Tor is not valid. Extracting...");
        extract_tor()?;
    }

    #[cfg(feature="snowflake")]
    if !is_snowflake_binary_valid().unwrap_or(false) {
        error!("Snowflake is not valid. Extracting...");
        extract_tor()?;
    }

    Ok(())
}


/// Extracts the tor binary from the assets into the `TOR_BINARY_PATH`
fn extract_tor() -> Result<()> {

    #[cfg(all(target_os ="windows", target_arch = "x86_64"))]
    let tor_zip = include_bytes!(concat!(env!("OUT_DIR"), "/windows/x86_64/tor.zip"));

    #[cfg(all(target_os ="windows", target_arch = "x86", not(target_arch="x86_64")))]
    let tor_zip = include_bytes!(concat!(env!("OUT_DIR"), "/windows/i686/tor.zip"));

    #[cfg(all(target_os ="linux", target_arch = "x86_64"))]
    let tor_zip = include_bytes!(concat!(env!("OUT_DIR"), "/linux/x86_64/tor.zip"));

    #[cfg(all(target_os ="linux", target_arch = "x86", not(target_arch="x86_64")))]
    let tor_zip = include_bytes!(concat!(env!("OUT_DIR"), "/windows/i686/tor.zip"));

    let target_dir = get_root_dir();
    zip_extract::extract(Cursor::new(tor_zip),&target_dir, true)?;

    Ok(())
}

/// Checks if the tor binary is valid by comparing the hash of the binary to the hash in `TOR_BINARY_HASH`
/// 
/// # Returns
/// 
/// a boolean indicating whether the tor binary is valid or not (has a valid hash)
fn is_tor_binary_valid() -> Result<bool> {
    let mut file = File::open(TOR_BINARY_PATH.clone())?;

    // create a Sha256 object
    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher)?;

    // read hash digest and consume hasher
    let result = hasher.finalize();
    let result_hex = hex::encode(result);

    Ok(result_hex == TOR_BINARY_HASH.clone())
}



/// Checks if the tor binary is valid by comparing the hash of the binary to the hash in `TOR_BINARY_HASH`
/// 
/// # Returns
/// 
/// a boolean indicating whether the tor binary is valid or not (has a valid hash)
#[cfg(feature="snowflake")]
fn is_snowflake_binary_valid() -> Result<bool> {
    use crate::consts::{SNOWFLAKE_BINARY_HASH, get_snowflake_path};

    let mut file = File::open(get_snowflake_path())?;

    // create a Sha256 object
    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher)?;

    // read hash digest and consume hasher
    let result = hasher.finalize();
    let result_hex = hex::encode(result);

    Ok(result_hex == SNOWFLAKE_BINARY_HASH.clone())
}

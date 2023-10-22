use std::{fs::File, io::{self, Write}};

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::tor::consts::{TOR_ZIP_PATH, TOR_ZIP_HASH};


/**
Checks for the binary of tor at TOR_BINARY_PATH and extracts the tor
binary if the file does not exist or the hash is wrong
*/
pub fn check_integrity() -> Result<()> {
    let is_valid = TOR_ZIP_PATH.is_file() && is_tor_binary_valid().unwrap_or(false);

    if !is_valid {
        extract_tor()?;
    }

    Ok(())
}



fn extract_tor() -> Result<()> {

    #[cfg(all(target_os ="windows", target_arch = "x86_64"))]
    let tor_zip = include_bytes!("../../assets/windows/x86_64/tor.zip");

    #[cfg(all(target_os ="windows", target_arch = "x86", not(target_arch="x86_64")))]
    let tor_zip = include_bytes!("../../assets/windows/i686/tor.zip");

    #[cfg(all(target_os ="linux", target_arch = "x86_64"))]
    let tor_zip = include_bytes!("../../assets/linux/x86_64/tor.zip");

    #[cfg(all(target_os ="linux", target_arch = "x86", not(target_arch="x86_64")))]
    let tor_zip = include_bytes!("../../assets/windows/i686/tor.zip");

    let mut f = File::create(TOR_ZIP_PATH.clone())?;
    f.write_all(tor_zip)?;

    Ok(())
}


fn is_tor_binary_valid() -> Result<bool> {
    let mut file = File::open(TOR_ZIP_PATH.clone())?;

    // create a Sha256 object
    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher)?;

    // read hash digest and consume hasher
    let result = hasher.finalize();
    let result_hex = hex::encode(result);

    Ok(result_hex == TOR_ZIP_HASH.clone())
}

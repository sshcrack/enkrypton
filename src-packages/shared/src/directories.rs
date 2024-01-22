use std::{env::current_exe, path::{PathBuf, Path}, fs::{create_dir_all, self}, ffi::OsString};

use anyhow::Result;

/// The main directory of enkrypton, creating it if it does not exist
///
/// # Returns
///
/// Returns the path of the root directory
pub fn get_root_dir() -> PathBuf {
    let mut buf = current_exe().unwrap().parent().unwrap().to_path_buf();
    buf.push("enkrypton_root/");

    if !buf.is_dir() {
        create_dir_all(&buf).unwrap();
    }

    buf
}


/// The main root directory of tor, creating it if it does not exist
///
/// # Returns
///
/// Returns the path of the root directory
pub fn get_tor_dir() -> PathBuf {
    let mut buf = get_root_dir();
    buf.push("tor");

    if buf.is_file() {
        fs::remove_file(&buf).unwrap();
    }

    if !buf.is_dir() {
        create_dir_all(&buf).unwrap();
    }

    buf
}

/// The service directory the tor proxy should be using. Defaulted to `enkrypton_root/service`.
/// Creates a directory if it does not exist
///
/// # Returns
///
/// A path to the service directory
pub fn get_service_dir() -> Result<OsString> {
    let mut dir = get_root_dir();
    dir.push("service");

    if !dir.is_dir() {
        fs::create_dir(&dir)?;
    }

    return Ok(dir.into_os_string());
}

/// The data directory tor should use. This is `enkrypton_root/data` for now.
/// Creates if it does not exist
///
/// # Returns
///
/// A path to the data directory
pub fn get_data_dir() -> Result<OsString> {
    let mut dir = get_root_dir();
    dir.push("data");

    if !dir.is_dir() {
        fs::create_dir(&dir)?;
    }

    return Ok(dir.into_os_string());
}


/// This file is the "settings" file for the tor proxy.
/// More info [here](https://manpages.debian.org/jessie/tor/torrc.5).
///
/// # Returns
///
/// A path to the `torrc` file.
pub fn get_torrc() -> PathBuf {
    let mut dir = get_root_dir();
    dir.push("torrc");

    return dir;
}

/// The path to the tor executable. This is `enkrypton_root/tor.exe` for windows and `enkrypton_root/tor` for linux.
///
/// # Returns
///
/// The platform specific path to the tor executable
pub fn get_tor_path() -> PathBuf {
    let tor_write_path = get_tor_dir();
    #[cfg(target_os="windows")]
    return tor_write_path.join("tor.exe");

    #[cfg(target_os="linux")]
    return tor_write_path.join("tor");
}

/// The path to the storage file (where user data is encrypted and stored).
///
/// # Returns
///
/// The path to the binary file
pub fn get_storage_path() -> Box<Path> {
    let mut root = get_root_dir();
    root.push("storage.bin");

    return root.into_boxed_path();
}
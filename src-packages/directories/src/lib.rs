
pub fn get_root_dir() -> PathBuf {
    let mut buf = current_exe().unwrap().parent().unwrap().to_path_buf();
    buf.push("enkrypton_root/");

    if !buf.is_dir() {
        create_dir_all(&buf).unwrap();
    }

    buf
}


fn get_service_dir() -> Result<OsString> {
    let mut dir = get_root_dir();
    dir.push("service");

    if !dir.is_dir() {
        fs::create_dir(&dir)?;
    }

    return Ok(dir.into_os_string());
}

fn get_data_dir() -> Result<OsString> {
    let mut dir = get_root_dir();
    dir.push("data");

    if !dir.is_dir() {
        fs::create_dir(&dir)?;
    }

    return Ok(dir.into_os_string());
}


/// The file to tell tor what to do (https://manpages.debian.org/jessie/tor/torrc.5)
pub fn get_torrc() -> PathBuf {
    let mut dir = get_root_dir();
    dir.push("torrc");

    return dir;
}

/// Path to the extracted tor binary, again this is platform specific
fn get_tor_path() -> PathBuf {
    let tor_write_path = get_root_dir();
    #[cfg(target_os="windows")]
    return tor_write_path.join("tor.exe");

    #[cfg(target_os="linux")]
    return tor_write_path.join("tor");
}

pub fn get_storage_path() -> Box<Path> {
    let mut root = get_root_dir();
    root.push("storage.bin");

    return root.into_boxed_path();
}
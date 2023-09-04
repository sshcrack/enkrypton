use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use anyhow::Result;

use crate::tor::config::CONFIG;

//TODO make non blocking and into async func
pub fn get_service_hostname() -> Result<Option<String>> {
    let dir = &CONFIG.service_dir;
    let mut hostname_path = PathBuf::from(dir);
    hostname_path.push("hostname");

    if !hostname_path.is_file() {
        return Ok(None);
    }

    let file = File::open(hostname_path)?;
    let mut reader = BufReader::new(file);

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    buffer = buffer
        .trim_matches(|c: char| {
            return !c.is_ascii_alphanumeric() && !c.is_ascii_punctuation();
        })
        .replace(".onion", "")
        .to_string();
    return Ok(Some(buffer));
}

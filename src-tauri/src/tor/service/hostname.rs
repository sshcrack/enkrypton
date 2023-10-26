use std::path::PathBuf;

use anyhow::Result;
use tokio::{fs::File, io::{BufReader, AsyncReadExt}};

use crate::tor::config::CONFIG;

pub async fn get_service_hostname(client: bool) -> Result<Option<String>> {
    let dir = &CONFIG.service_dir();
    let mut hostname_path = PathBuf::from(dir);
    hostname_path.push("hostname");

    if !hostname_path.is_file() {
        return Ok(None);
    }

    let file = File::open(hostname_path).await?;
    let mut reader = BufReader::new(file);

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await?;

    buffer = buffer
        .trim_matches(|c: char| {
            return !c.is_ascii_alphanumeric() && !c.is_ascii_punctuation();
        })
        .replace(".onion", "")
        .to_string();

    #[cfg(feature="dev")]
    {
        buffer = format!("{}-dev-{}", buffer, if client { "client"} else { "server" });
    }

    return Ok(Some(buffer));
}

use std::path::PathBuf;

use anyhow::Result;
use shared::config::CONFIG;
use tokio::{fs::File, io::{BufReader, AsyncReadExt}};


/// Gets the current hostname of our tor service
///
/// # Arguments
///
/// **DEV ONLY**
/// * `_client` - Used for dev purposes only, so you can send messages to yourself
///
/// # Returns
///
/// The current service hostname, `None` if the hostname could not be found
pub async fn get_service_hostname(_client: bool) -> Result<Option<String>> {
    let dir = &CONFIG.service_dir();
    let mut hostname_path = PathBuf::from(dir);
    hostname_path.push("hostname");

    if !hostname_path.is_file() {
        return Ok(None);
    }

    // Reads the hostname file
    let file = File::open(hostname_path).await?;
    let mut reader = BufReader::new(file);

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await?;

    // Trimming excess
    buffer = buffer
        .trim_matches(|c: char| {
            return !c.is_ascii_alphanumeric() && !c.is_ascii_punctuation();
        })
        .replace(".onion", "")
        .to_string();

    // Used to message self on development
    #[cfg(feature="dev")]
    {
        buffer = format!("{}-dev-{}", buffer, if _client { "client"} else { "server" });
    }

    return Ok(Some(buffer));
}

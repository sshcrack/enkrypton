use std::ffi::OsString;

#[cfg(target_family="unix")]
use std::fs::Permissions;

#[cfg(target_family="unix")]
use std::fs;

#[cfg(target_family="unix")]
use smol::fs::unix::PermissionsExt;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use port_check::free_local_port;

use crate::{get_data_dir, get_service_dir};

/// Contains the configuration for the Tor process
/// Such as the port, data dir, service dir, etc.
#[derive(Debug, Clone)]
pub struct TorConfig {
    /// The socks port tor is listening on
    socks_port: u16,

    /// The directory to store service_dir in
    service_dir: OsString,
    /// The port of the service
    service_port: u16,

    /// The directory to store tor data in
    data_dir: OsString,
}

impl TorConfig {
    /// Creates a new TorConfig
    /// 
    /// # Returns
    /// 
    /// The constructed TorConfig
    fn new() -> Result<Self> {
        // Checks for a free local port and stores it
        let socks_port = free_local_port().ok_or(anyhow!("Could not find a free port."))?;

        // The service directory tor should use
        let service_dir = get_service_dir()?;

        #[cfg(target_family = "unix")]
        // We are restricting the permissions of the service dir
        fs::set_permissions(&service_dir, Permissions::from_mode(0o700)).unwrap();

        // And the data directory
        let data_dir = get_data_dir()?;
        // And finding a free port for the service
        let service_port =
            free_local_port().ok_or(anyhow!("Could not find a free service port."))?;

        // Actually constructing this config struct
        return Ok(Self {
            socks_port,
            data_dir,
            service_dir,
            service_port,
        });
    }

    /// # Returns
    /// 
    /// the socks proxy port that tor should/is listening on
    pub fn socks_port(&self) -> u16 {
        self.socks_port
    }

    /// The web server port that we are listening on
    pub fn service_port(&self) -> u16 {
        self.service_port
    }

    /// The service directory that tor should use
    pub fn service_dir(&self) -> &OsString {
        return &self.service_dir;
    }

    /// The host the tor proxy should be listening on
    pub fn get_socks_host(&self) -> String {
        return format!("127.0.0.1:{}", self.socks_port);
    }

    /// Returns the hidden service host, as the name suggests
    pub fn get_hidden_service_host(&self) -> String {
        return format!("127.0.0.1:{}", self.service_port);
    }

    //noinspection SpellCheckingInspection
    /// Converts the configuration to a `torrc` file format
    ///
    /// # Returns
    ///
    /// The `torrc` file as a string
    pub fn to_text(&self) -> String {
        return format!(
"SocksPort {}
HiddenServiceDir \"{}\"
HiddenServicePort 80 {}
DataDirectory \"{}\"",
            self.get_socks_host(),
            self.service_dir.to_string_lossy().replace("\\", "/"),
            self.get_hidden_service_host(),
            self.data_dir.to_string_lossy().replace("\\", "/")
        );
    }
}

lazy_static! {
    /// The global tor configuration
    pub static ref CONFIG: TorConfig = TorConfig::new().unwrap();
}

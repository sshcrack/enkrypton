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

#[derive(Debug, Clone)]
pub struct TorConfig {
    socks_port: u16,

    service_dir: OsString,
    service_port: u16,

    data_dir: OsString,
}

impl TorConfig {
    fn new() -> Result<Self> {
        let socks_port = free_local_port().ok_or(anyhow!("Could not find a free port."))?;

        let service_dir = get_service_dir()?;

        #[cfg(target_family = "unix")]
        fs::set_permissions(&service_dir, Permissions::from_mode(0o700)).unwrap();

        let data_dir = get_data_dir()?;
        let service_port =
            free_local_port().ok_or(anyhow!("Could not find a free service port."))?;

        return Ok(Self {
            socks_port,
            data_dir,
            service_dir,
            service_port,
        });
    }

    pub fn socks_port(&self) -> u16 {
        self.socks_port
    }

    pub fn service_port(&self) -> u16 {
        self.service_port
    }

    pub fn service_dir(&self) -> &OsString {
        return &self.service_dir;
    }

    pub fn get_socks_host(&self) -> String {
        return format!("127.0.0.1:{}", self.socks_port);
    }

    pub fn get_hidden_service_host(&self) -> String {
        return format!("127.0.0.1:{}", self.service_port);
    }

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
    pub static ref CONFIG: TorConfig = TorConfig::new().unwrap();
}

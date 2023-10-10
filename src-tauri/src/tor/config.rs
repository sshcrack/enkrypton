use std::{ffi::OsString, fs};

use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use port_check::free_local_port;

use crate::{client::Client, util::get_root_dir};

#[derive(Debug, Clone)]
pub struct TorConfig {
    socks_port: u16,

    service_dir: OsString,
    service_port: u16,
}

impl TorConfig {
    fn new() -> Result<Self> {
        let socks_port = free_local_port().ok_or(anyhow!("Could not find a free port."))?;

        let service_dir = get_service_dir()?;
        let service_port = free_local_port().ok_or(anyhow!("Could not find a free service port."))?;

        return Ok(Self {
            socks_port,
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
HiddenServicePort 80 {}",
            self.get_socks_host(),
            self.service_dir.to_string_lossy().replace("\\", "/"),
            self.get_hidden_service_host()
        );
    }
}

lazy_static! {
    pub static ref CONFIG: TorConfig =  TorConfig::new().unwrap();
    pub static ref TOR_CLIENT: Client = Client::from_config().unwrap();
}

fn get_service_dir() -> Result<OsString> {
    let mut dir = get_root_dir();
    dir.push("service");

    if !dir.is_dir() {
        fs::create_dir(&dir)?;
    }

    return Ok(dir.into_os_string());
}
/*
fn random_pass(length: usize) -> String {
    let chars: Vec<&str> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .split("")
        .collect();

    let mut rng = rand::thread_rng();
    let mut passwd = "".to_string();
    for _ in 0..length {
        let index = rng.gen_range(0..chars.len());
        passwd = passwd + chars.get(index).unwrap().to_owned();
    }

    return passwd.to_string();
}
*/

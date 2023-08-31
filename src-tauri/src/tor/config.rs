use std::{ffi::OsString, fs};

use anyhow::Result;
use lazy_static::lazy_static;

use crate::client::Client;

use super::consts::get_tor_dir;

#[derive(Debug, Clone)]
pub struct TorConfig {
    pub socks_port: u32,

    pub service_dir: OsString,
    pub service_port: u16,
}

impl TorConfig {
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
    pub static ref CONFIG: TorConfig = TorConfig {
        socks_port: 14569,

        service_dir: get_service_dir().unwrap(),
        service_port: 5467
    };
    pub static ref TOR_CLIENT: Client = Client::from_config().unwrap();
}

fn get_service_dir() -> Result<OsString> {
    let mut dir = get_tor_dir();
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

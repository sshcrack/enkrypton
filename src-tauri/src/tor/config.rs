use std::{path::PathBuf, fs};

use anyhow::Result;
use lazy_static::lazy_static;
use rand::Rng;
use reqwest::{Client, Proxy};

use super::consts::get_tor_dir;

#[derive(Debug, Clone)]
pub struct TorConfig {
    socks_port: u32,

    service_dir: PathBuf,
    service_port: u32,
}

impl TorConfig {
    pub fn to_text(&self) -> String {
        return format!(
"SocksPort 127.0.0.1:{}
HiddenServiceDir \"{}\"
HiddenServicePort 80 127.0.0.1:{}",
            self.socks_port,
            self.service_dir.to_string_lossy().replace("\\", "/"),
            self.service_port
        );
    }

    pub fn create_client(&self) -> Result<Client> {
        let proxy = Proxy::https(format!("socks5h://127.0.0.1:{}", self.socks_port))?;
        let res = Client::builder().proxy(proxy).build()?;
        return Ok(res);
    }
}

lazy_static! {
    pub static ref CONFIG: TorConfig = TorConfig {
        socks_port: 14569,

        service_dir: get_service_dir(),
        service_port: 5467
    };

    pub static ref CLIENT: Client = CONFIG.create_client().unwrap();
}

fn get_service_dir() -> PathBuf {
    let mut dir = get_tor_dir();
    dir.push("service");

    if !dir.is_dir() {
        fs::create_dir(&dir).unwrap();
    }

    return dir;
}

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

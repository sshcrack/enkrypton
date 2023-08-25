use std::path::PathBuf;

use anyhow::Result;
use lazy_static::lazy_static;
use rand::Rng;
use reqwest::{Client, Proxy};

use super::consts::get_tor_dir;

#[derive(Debug, Clone)]
struct TorConfig {
    https_port: u32,

    https_username: String,
    https_password: String,

    service_dir: PathBuf,
    service_port: u32,
}

impl TorConfig {
    pub fn to_text(&self) -> String {
        return format!(
            "
        HTTPSProxy 127.0.0.1:{}\n
        HTTPSProxyAuthenticator {}:{}\n
        HiddenServiceDir {},\n
        HiddenServicePort 80 127.0.0.0.1:{}\n
        ",
            self.https_port,
            self.https_username,
            self.https_password,
            self.service_dir.to_string_lossy(),
            self.service_port
        );
    }

    pub fn create_client(&self) -> Result<Client> {
        let proxy = Proxy::https(format!("https://127.0.0.1:{}", self.https_port))?
            .basic_auth(&self.https_username, &self.https_password);

        let res = Client::builder().proxy(proxy).build()?;
        return Ok(res);
    }
}

lazy_static! {
    pub static ref CONFIG: TorConfig = TorConfig {
        https_port: 14569,
        https_username: "admin".to_string(),
        https_password: random_pass(20),

        service_dir: get_tor_dir(),
        service_port: 5467
    };

    pub static ref CLIENT: Client = CONFIG.create_client().unwrap();
}

fn get_service_dir() -> PathBuf {
    let mut dir = get_tor_dir();
    dir.push("service");
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

use anyhow::{anyhow, Result};

use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use url::Url;

use crate::{tor::config::CONFIG, util::get_servername};

#[derive(Debug)]
pub struct SocksProxy {
    auth: Option<(String, String)>,
    proxy_url: Url,
}

impl SocksProxy {
    pub fn new() -> Result<Self> {
        let addr = format!("socks5://127.0.0.1:{}", CONFIG.socks_port);

        let url = Url::parse(&addr)?;
        let scheme = url.scheme();
        if scheme != "socks5" {
            return Err(anyhow!("Scheme is not 'socks5'"));
        }

        let username = url.username();
        let password = url.password();

        let mut auth = None as Option<(String, String)>;
        if let Some(passwd) = password {
            if !username.is_empty() {
                auth = Some((username.to_string(), passwd.to_string()));
            }
        }

        Ok(SocksProxy {
            auth,
            proxy_url: url,
        })
    }

    pub async fn connect(&self, destination_url: &Url) -> Result<Socks5Stream<TcpStream>> {
        let proxy_url = get_servername(&self.proxy_url)?;
        let dest_server = get_servername(destination_url)?;

        println!(
            "Connecting to {} with server_name {}",
            proxy_url, dest_server
        );
        if let Some((username, password)) = self.auth.as_ref() {
            Ok(
                Socks5Stream::connect_with_password(&*proxy_url, dest_server, username, password)
                    .await?,
            )
        } else {
            Ok(Socks5Stream::connect(&*proxy_url, dest_server).await?)
        }
    }
}

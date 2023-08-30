use std::net::TcpStream;

use anyhow::{anyhow, Result};
use tokio_socks::tcp::Socks5Stream;
use url::Url;

#[derive(Debug)]
pub struct SocksProxy {
    auth: Option<(String, String)>,
    proxy_url: Url,
}

impl SocksProxy {
    pub fn new(addr: &str) -> Result<Self> {
        let url = Url::parse(addr)?;
        let scheme = url.scheme();
        if scheme != "socks5" {
            return Err(anyhow!("Scheme is not 'socks5'"));
        }

        let username = url.username();
        let password = url.password();

        let mut auth = None as Option<(String, String)>;
        if !username.is_empty() && password.is_some() {
            let password = password.unwrap();

            auth = Some((username.to_string(), password.to_string()));
        }

        Ok(SocksProxy {
            auth,
            proxy_url: url,
        })
    }

    pub async fn connect(&self, server_name: &str) -> Result<TcpStream> {
        let host = self.proxy_url.host_str();
        let port = self.proxy_url.port();
        if host.is_none() || port.is_none() {
            return Err(anyhow!(
                "Host or port is none, can't connect (full url is {}, host {:?}, port {:?})",
                self.proxy_url.to_string(),
                host,
                port
            ));
        }

        let proxy_url = &*format!("{}:{}", host.unwrap(), port.unwrap());

        println!("Connecting to {}", proxy_url);
        let tokio_stream = if let Some((username, password)) = self.auth.as_ref() {
            Socks5Stream::connect_with_password(proxy_url, server_name, username, password).await?
        } else {
            Socks5Stream::connect(proxy_url, server_name).await?
        };

        let inner = tokio_stream.into_inner();
        let std = inner.into_std()?;

        std.set_nonblocking(false);
        return Ok(std);
    }
}

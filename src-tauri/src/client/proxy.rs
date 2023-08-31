use anyhow::{anyhow, Result};
use tokio_socks::tcp::Socks5Stream;
use tokio::net::TcpStream;
use url::Url;

use crate::tor::config::CONFIG;

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

    pub async fn connect(&self, server_name: &str) -> Result<Socks5Stream<TcpStream>> {
        let host = self.proxy_url.host_str()
            .ok_or(anyhow!("Host is not in the proxy url ({})", self.proxy_url))?;

        let port = self.proxy_url.port()
            .ok_or(anyhow!("Port is not in the proxy url ({})", self.proxy_url))?;

        let proxy_url = &*format!("{}:{}", host, port);

        println!("Connecting to {}", proxy_url);
        let tokio_stream = if let Some((username, password)) = self.auth.as_ref() {
            Socks5Stream::connect_with_password(proxy_url, server_name, username, password).await?
        } else {
            Socks5Stream::connect(proxy_url, server_name).await?
        };

        return Ok(tokio_stream);
    }
}

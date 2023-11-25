use anyhow::{anyhow, Result};

use shared::config::CONFIG;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use url::Url;

use crate::client::util::get_servername;

/// A socks proxy which is used to connect to the tor network
#[derive(Debug)]
pub struct SocksProxy {
    /// The authentication to use for the proxy
    auth: Option<(String, String)>,
    /// The url of the proxy
    proxy_url: Url,
}

impl SocksProxy {
    /// Creates a new socks proxy from the config
    pub fn new() -> Result<Self> {
        let addr = format!("socks5://127.0.0.1:{}", CONFIG.socks_port());

        // Parses the url and checks if the scheme is socks5
        let url = Url::parse(&addr)?;
        let scheme = url.scheme();
        if scheme != "socks5" {
            return Err(anyhow!("Scheme is not 'socks5'"));
        }

        // Checks if the url has a username and password
        let username = url.username();
        let password = url.password();

        // If the password is set, the username must be set too
        let mut auth = None as Option<(String, String)>;
        if let Some(passwd) = password {
            if !username.is_empty() {
                auth = Some((username.to_string(), passwd.to_string()));
            }
        }

        // Returns the proxy
        Ok(SocksProxy {
            auth,
            proxy_url: url,
        })
    }

    /// Connects to the destination url over the tor network
    pub async fn connect(&self, destination_url: &Url) -> Result<Socks5Stream<TcpStream>> {
        // Getting required urls
        let proxy_url = get_servername(&self.proxy_url)?;
        let dest_server = get_servername(destination_url)?;

        println!(
            "[PROXY] Connecting to {} with server_name {}",
            proxy_url, dest_server
        );

        // And connecting with or without authentication
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

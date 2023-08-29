use std::net::TcpStream;

use anyhow::{anyhow, Result};
use tokio_socks::tcp::Socks5Stream;
use url::Url;

#[derive(Debug)]
pub struct SocksProxy {
    auth: Option<(String, String)>,
    url: Url,
    stream: Option<TcpStream>,
}

impl SocksProxy {
    /** checks if the current stream is alive */
    fn is_alive(&self) -> bool {
        if self.stream.is_none() {
            return false;
        }

        let unwrapped = self.stream.as_ref().unwrap();
        let mut buf = [];

        let bytes_read = unwrapped.peek(&mut buf);

        let is_ready = bytes_read.is_ok_and(|e| e != 0);

        return !is_ready;
    }

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
            url,
            stream: None,
        })
    }

    pub async fn get_or_connect_socket(&self, addr: &str) {

    }

    pub async fn connect(&self, addr: &str) -> Result<Socks5Stream<TcpStream>> {
        let host = self.url.host_str();
        let port = self.url.port();
        if host.is_none() || port.is_none() {
            return Err(anyhow!(
                "Host or port is none, can't connect (full url is {}, host {:?}, port {:?})",
                self.url.to_string(),
                host,
                port
            ));
        }

        let destination = &*format!("{}:{}", host.unwrap(), port.unwrap());
        if let Some((username, password)) = self.auth.as_ref() {
            let stream =
                Socks5Stream::connect_with_password(destination, addr, username, password).await?;

            return Ok(stream);
        }

        println!("Connecting to {}", destination);

        let stream = Socks5Stream::connect(destination, addr).await?;
        return Ok(stream);
    }
}

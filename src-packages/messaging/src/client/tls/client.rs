use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::net::TcpStream;
use tokio_rustls::{
    client::TlsStream,
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
    TlsConnector,
};
use tokio_socks::tcp::Socks5Stream;

use url::Url;

use crate::client::SocksProxy;

use super::request::Request;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/117.0";

/// A web client which is used to send requests to the server over the tor network
#[derive(Debug)]
pub struct WebClient {
    /// The proxy used to connect to the tor network
    proxy: SocksProxy,
}

impl WebClient {
    /// Gets the underlying SocksProxy of this client
    /// # Returns
    ///
    /// The proxy used to connect to the tor network
    pub(super) fn proxy(&self) -> &SocksProxy {
        &self.proxy
    }

    /// Creates a new web client from the config with the default tor proxy port
    /// # Returns
    ///
    /// Creates a new web client from the config with the default tor proxy port
    pub fn from_config() -> Result<Self> {
        let tor_proxy = SocksProxy::new()?;

        Ok(WebClient { proxy: tor_proxy })
    }

    /// Just just opens a connection to the proxy, uses it and closes it after that.
    /// Could be optimized but not doing that for literally one connection.
    /// Oh also: blocking so use in threads thanks
    ///
    /// # Arguments
    ///
    /// * `addr` - The url to send a get request to
    ///
    /// # Returns
    ///
    /// The struct that can be used to send the request
    pub fn get(&self, addr: &str) -> Request<'_> {
        Request::from_client(self, "GET", addr)
            .header("User-Agent", USER_AGENT)
            .header("Accept", "*/*")
    }

    /// The tls config to use for the client
    ///
    /// # Returns
    ///
    /// The TLS Configuration
    fn get_tls_config(&self) -> ClientConfig {
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth()
    }

    //noinspection SpellCheckingInspection
    /// Creates a connection to the given url using the proxy
    ///
    /// # Arguments
    ///
    /// * `proxy` - Proxy to use when connecting
    /// * `url` - The url to connect to
    ///
    /// # Returns
    /// Returns the connection to the given url
    ///
    pub(super) async fn create_connection(
        &self,
        proxy: Socks5Stream<TcpStream>,
        url: &Url,
    ) -> Result<TlsStream<Socks5Stream<TcpStream>>> {
        let config = self.get_tls_config();

        let server_name_raw = url
            .host_str()
            .ok_or(anyhow!("Url has to have a host."))?
            .to_string();
        let server_name: ServerName = server_name_raw.try_into()?;
        let connector = TlsConnector::try_from(Arc::new(config))?;

        // and connecting it to the proxy
        let stream = connector.connect(server_name, proxy).await?;

        Ok(stream)
    }
}

use std::{sync::Arc};

use anyhow::{Result};
use async_rustls::{client::TlsStream, TlsConnector};
use rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
use smol::net::TcpStream;
use tokio::net::TcpStream as TokioTcpStream;
use tokio_socks::tcp::Socks5Stream;

use webpki_roots::TLS_SERVER_ROOTS;

use crate::{client::SocksProxy, tor::config::TorConfig};

use super::request::Request;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/117.0";

#[derive(Debug)]
pub struct Client {
    pub(super) proxy: SocksProxy,
}

impl Client {
    pub fn from_config(config: &TorConfig) -> Result<Self> {
        let tor_addr = format!("socks5://127.0.0.1:{}", config.socks_port);
        let tor_proxy = SocksProxy::new(&tor_addr)?;

        Ok(Client { proxy: tor_proxy })
    }

    /** Just just opens a connection to the proxy, uses it and closes it after that.
     * Coooouuld be optimized but not doing that for literally one connection
     * Oh also: blocking so use in threads thanks
     */
    pub fn get(&self, addr: &str) -> Request {
        Request::from_client(self, "GET", addr)
            .header("User-Agent", USER_AGENT)
            .header("Accept", "*/*")
    }

    fn get_root_store(&self) -> RootCertStore {
        let mut root_store = RootCertStore::empty();
        root_store.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        return root_store;
    }

    fn get_tls_config(&self) -> ClientConfig {
        let root_store = self.get_root_store();

        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth()
    }

    pub(super) async fn create_connection(
        &self,
        proxy: Socks5Stream<TokioTcpStream>,
        server_name_raw: &str,
    ) -> Result<TlsStream<TcpStream>> {
        let config = self.get_tls_config();

        let server_name: ServerName = server_name_raw.try_into()?;
        let connector = TlsConnector::try_from(Arc::new(config))?;

        // converting the streams
        let std = proxy.into_inner().into_std()?;
        let smol_proxy = TcpStream::try_from(std)?;

        let stream = connector.connect(server_name, smol_proxy).await?;

        return Ok(stream);
    }
}

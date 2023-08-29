use std::{net::TcpStream, sync::Arc, io::Write};

use anyhow::{anyhow, Result};
use rustls::{ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore};
use tauri::async_runtime::block_on;
use url::Url;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::{client::SocksProxy, tor::config::TorConfig};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/117.0";

#[derive(Debug)]
pub struct Client {
    proxy: SocksProxy,
}

impl Client {
    pub fn from_config(config: &TorConfig) -> Result<Self> {
        let tor_addr = format!("127.0.0.1:{}", config.socks_port);
        let tor_proxy = SocksProxy::new(&tor_addr)?;

        Ok(Client { proxy: tor_proxy })
    }


    /** Just just opens a connection to the proxy, uses it and closes it after that.
     * Coooouuld be optimized but not doing that for literally one connection
     * Oh also: blocking so use in threads thanks
     */
    pub fn get(&self, addr: &str) -> Result<()> {
        let url = Url::parse(addr)?;
        let path = url.path();
        let server_name = url
            .host_str()
            .ok_or(anyhow!("Expected the url to have a host name"))?;

        let stream = block_on(self.proxy.connect(addr))?;
        let stream = stream.into_inner().into_std()?;

        stream.set_nonblocking(false);

        // setting up trusted certificate stuff don't ask about it
        let mut root_store = RootCertStore::empty();
        root_store.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let mut c_conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
        let mut tls = rustls::Stream::new(&mut c_conn, &mut sock);

        tls.write_all(format!("GET {} HTTP/1.1\r\n", path).as_bytes());

        let headers = Vec::<String>::new();
        headers.push(format!("Host: {}\r\n"))

        let to_write = format!(
            "GET {} HTTP/1.1\r\n" + "Host: {}\r\n",
            path, server_name, USER_AGENT
        );

        tls.write_all(to_write.as_bytes()).unwrap();
        let ciphersuite = tls.conn.negotiated_cipher_suite().unwrap();

        let mut plaintext = Vec::new();
        tls.read_to_end(&mut plaintext).unwrap();
        stdout().write_all(&plaintext).unwrap();
        Ok(())
    }
}

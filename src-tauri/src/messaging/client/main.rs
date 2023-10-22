use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    messaging::payloads::{WsClientStatus, WsClientUpdate, WsMessagePayload},
    util::{get_app, url::UrlOnion},
};
use anyhow::{anyhow, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, error, warn};
use tauri::{async_runtime::block_on, Manager};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use url::Url;

use super::SocksProxy;

type WriteStream = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
type ReadStream = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct MessagingClient {
    write: Arc<Mutex<WriteStream>>,
    read: Arc<Mutex<ReadStream>>,
    url: Url,
}

impl MessagingClient {
    pub async fn new(onion_hostname: &str) -> Result<Self> {
        let onion_addr = format!("ws://{}.onion/ws/", onion_hostname);

        debug!("Creating proxy...");
        let proxy = SocksProxy::new()?;
        debug!("Connecting Proxy...");
        let mut onion_addr = Url::parse(&onion_addr)?;
        onion_addr
            .set_scheme("ws")
            .or(Err(anyhow!("Could not set scheme")))?;

        let sock = proxy.connect(&onion_addr).await?;

        debug!("Connecting Tungstenite...");
        let (ws_stream, _) = tokio_tungstenite::client_async(&onion_addr, sock).await?;

        let (write, read) = ws_stream.split();
        return Ok(MessagingClient {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            url: onion_addr,
        });
    }

    pub async fn send_msg_txt(&self, msg: &str) -> Result<()> {
        return self.send_msg(Message::Text(msg.to_string())).await;
    }

    pub async fn send_msg(&self, msg: Message) -> Result<()> {
        debug!("Locking write mutex...");
        let mut state = self.write.lock().await;
        state.send(msg).await?;

        Ok(())
    }
}

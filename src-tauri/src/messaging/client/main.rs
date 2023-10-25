use std::{
    sync::Arc, thread::{self, JoinHandle},
};
use anyhow::{anyhow, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, warn};
use tauri::async_runtime::block_on;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use url::Url;
use crate::messaging::{packages::C2SPacket, HEARTBEAT};

use super::SocksProxy;

type WriteStream = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
type ReadStream = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;


#[derive(Debug, Clone)]
pub struct MessagingClient {
    write: Arc<Mutex<WriteStream>>,
    read: Arc<Mutex<ReadStream>>,
    url: Url,
    heartbeat_thread: Arc<Option<JoinHandle<()>>>
}

impl MessagingClient {
    pub async fn new(onion_hostname: &str) -> Result<Self> {
        debug!("Creating verify packet...");
        let verify_packet = C2SPacket::identity(onion_hostname).await?;

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

        let (mut write, read) = ws_stream.split();


        debug!("Sending verify packet");
        write.send(verify_packet.try_into()?).await?;

        let mut c = MessagingClient {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            url: onion_addr,
            heartbeat_thread: Arc::new(None)
        };

        debug!("Spawning heartbeat thread");
        c.spawn_heartbeat_thread();
        return Ok(c);
    }

    pub async fn send_packet(&self, msg: C2SPacket) -> Result<()> {
        debug!("Locking write mutex...");
        let mut state = self.write.lock().await;
        state.send(msg.try_into()?).await?;

        Ok(())
    }

    fn spawn_heartbeat_thread(&mut self) {
        if self.heartbeat_thread.is_none() {
            warn!("Could not spawn heartbeat thread, already exists");
            return;
        }

        let sender = self.write.clone();
        let handle = thread::spawn(move || {
            loop {
                thread::sleep(*HEARTBEAT);

                let mut temp = block_on(sender.lock());
                let temp = temp.send(Message::Ping(vec![]));
                let res = block_on(temp);

                if let Err(e) = res {
                    warn!("Could not send heartbeat: {:?}", e);
                }
            }
        });

        self.heartbeat_thread = Arc::new(Some(handle));
    }
}

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
    listen_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
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

        debug!("Getting app handle...");
        let app = get_app().await;

        let hostname = onion_addr.to_hostname().unwrap();

        app.emit_all(
            "ws_client_update",
            WsClientUpdate {
                hostname,
                status: WsClientStatus::CONNECTED,
            },
        )?;

        return Ok(MessagingClient {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            listen_thread: Arc::new(Mutex::new(None)),
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

    pub async fn create_listen_thread(&self) -> Result<()> {
        let read = self.listen_thread.try_lock()?;
        if read.is_some() {
            warn!("Tried to listen twice");
            return Ok(());
        }

        drop(read);

        let cloned = self.clone();
        let handle = thread::spawn(move || {
            let res = block_on(cloned.read_thread());
            if let Err(e) = res {
                error!("Could not listen: {}", e);
            }
        });

        debug!("Writing handle");
        let mut write = self.listen_thread.lock().await;
        write.replace(handle);

        Ok(())
    }

    pub async fn read_thread(&self) -> Result<()> {
        debug!("Locking read mutex...");
        let mut state = self.read.try_lock()?;

        let is_listening = Arc::new(AtomicBool::new(false));

        let is_listening_clone = is_listening.clone();
        let self_clone = self.clone();
        let handle = thread::spawn(move || {
            while is_listening_clone.load(Ordering::Relaxed) {
                let res = block_on(self_clone.send_msg(Message::Ping(Vec::new())));
                if res.is_err() {
                    error!("Could not send heartbeat: {}", res.unwrap_err());
                }

                thread::sleep(HEARTBEAT_INTERVAL);
            }
        });

        let self_clone = self.clone();
        state
            .by_ref()
            .for_each_concurrent(2, |msg| {
                let self_clone = self_clone.clone();
                async move {
                    let hostname = self.url.to_hostname().unwrap();
                    println!("Hostname is {}", hostname);

                    if let Err(e) = msg {
                        warn!("Read err: {}", e);
                        return;
                    }

                    let msg = msg.unwrap();
                    if msg.is_ping() {
                        let res = self_clone.send_msg(Message::Pong(Vec::new())).await;
                        error!("Could not send ping message: {}", res.unwrap_err());
                    }

                    if msg.is_text() {
                        let msg = msg.into_text().unwrap();
                        debug!("New Msg: {}", msg);

                        let handle = get_app().await;

                        let res = handle.emit_all(
                            &format!("msg-{}", hostname),
                            WsMessagePayload { message: msg },
                        );

                        if res.is_err() {
                            error!("Could not send msg payload: {}", res.unwrap_err());
                        }
                    } else {
                        debug!("Unknown msg {:#?}", msg);
                    }
                }
            })
            .await;

        let app = get_app().await;
        let hostname = self.url.to_hostname().unwrap();

        is_listening.store(false, Ordering::Release);
        debug!("Waiting for heartbeat handle to exit");

        let _ignore = handle.join();

        app.emit_all(
            "ws_client_update",
            WsClientUpdate {
                hostname,
                status: WsClientStatus::DISCONNECTED,
            },
        )?;
        Ok(())
    }
}

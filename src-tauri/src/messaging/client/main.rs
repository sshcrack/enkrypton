use crate::messaging::{
    packets::{C2SPacket, S2CPacket},
    HEARTBEAT, MESSAGING,
};
use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, warn};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use tauri::async_runtime::block_on;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use url::Url;

use super::{SocksProxy, manager_ext::ManagerExt};

type WriteStream = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
type ReadStream = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;

#[derive(Debug)]
pub struct MessagingClient {
    pub write: Arc<Mutex<WriteStream>>,

    receiver: String,
    heartbeat_thread: Arc<Option<JoinHandle<()>>>,
    read_thread: Arc<Option<JoinHandle<()>>>,

    pub rx: Receiver<S2CPacket>,
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

        let (tx, rx) = async_channel::unbounded();

        let arc_write =  Arc::new(Mutex::new(write));
        let mut c = MessagingClient {
            write: arc_write.clone(),
            heartbeat_thread: Arc::new(None),
            receiver: onion_hostname.to_string(),

            rx,
            read_thread: Arc::new(None)
        };

        debug!("Spawning heartbeat thread");
        c.spawn_heartbeat_thread();
        c.spawn_read_thread(tx, read, arc_write);

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
        let handle = thread::spawn(move || loop {
            thread::sleep(*HEARTBEAT);

            println!("Sending heartbeat...");
            let mut temp = block_on(sender.lock());
            let temp = temp.send(Message::Ping(vec![]));
            let res = block_on(temp);

            if let Err(e) = res {
                warn!("Could not send heartbeat: {:?}", e);
            }
        });

        self.heartbeat_thread = Arc::new(Some(handle));
    }

    fn spawn_read_thread(&mut self, tx: Sender<S2CPacket>, receiver: ReadStream, write: Arc<Mutex<WriteStream>>) {
        if self.read_thread.is_none() {
            warn!("Could not thread read thread, already exists");
            return;
        }

        let tmp = self.receiver.clone();

        let handle = thread::spawn(move || {
            let future = receiver.for_each_concurrent(2, |msg| {
                let receiver = tmp.clone();
                let write = write.clone();
                let tx = tx.clone();
                async move {
                    if msg.is_err() {
                        warn!("Could not parse client {:?}", msg.unwrap_err());
                        return;
                    }

                    let msg = msg.unwrap();
                    if !msg.is_binary() {
                        debug!("Received non binary message, returning");
                        return;
                    }

                    let bin = msg.into_data();
                    let packet = S2CPacket::try_from(&bin);
                    if let Err(e) = packet {
                        warn!("Could not parse packet {:?}", e);
                        return;
                    }

                    let packet = packet.unwrap();
                    let res = Self::handle_packet(packet, &receiver, write, tx).await;
                    if let Err(e) = res {
                        warn!("Could not handle packet: {:?}", e);
                        return;
                    }
                }
            });


            block_on(future);
        });

        self.read_thread = Arc::new(Some(handle));
    }

    async fn handle_packet(packet: S2CPacket, receiver: &str, write: Arc<Mutex<WriteStream>>, tx: Sender<S2CPacket>) -> Result<()> {
        match packet {
            S2CPacket::DisconnectMultipleConnections => todo!(),
            S2CPacket::VerifyIdentity(identity) => {
                identity.verify().await?;
                MESSAGING.write().await.set_remote_verified(receiver).await?;
            },
            S2CPacket::IdentityVerified => {
                MESSAGING.write().await.set_self_verified(receiver).await?;
                write.lock().await.send(C2SPacket::IdentityVerified.try_into()?).await?;
            },
            S2CPacket::Message(msg) => {
                // Redirecting msg to main handler
                tx.send(S2CPacket::Message(msg)).await?;
            },
        }

        Ok(())
    }
}

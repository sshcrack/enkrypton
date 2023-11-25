use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, info, warn, error};
use payloads::{
    event::AppHandleExt,
    packets::{C2SPacket, S2CPacket},
    payloads::{WsClientStatus, WsClientUpdatePayload, WsMessageStatus},
};
use shared::{get_app, name_struct, util::_get_name};
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread::{self, JoinHandle}
};
use tauri::async_runtime::block_on;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use url::Url;

use crate::{general::{IdentityProvider, IdentityVerify, MESSAGING}, client::{SocksProxy, manager_ext::ManagerExt, client::heartbeat::HeartbeatClient}};

use super::flush::FlushChecker;

/// The write stream of the websocket, just a wrapper
pub(super) type WriteStream = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
// The read stream of the websocket
pub(super) type ReadStream = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;

/// The client used to communicate with the server over the tor network
#[derive(Debug)]
pub struct MessagingClient {
    /// The stream to send messages to the server
    pub write: Arc<Mutex<WriteStream>>,

    /// The address the client is connected to
    receiver: String,
    /// The current heartbeat thread handle
    pub(super) heartbeat_thread: Arc<Option<JoinHandle<()>>>,
    /// The thread used to read messages from the server
    read_thread: Arc<Option<JoinHandle<()>>>,

    /// A receiver used in the common messaging manager to receive general messages
    pub rx: Receiver<S2CPacket>,
    /// The flush checker used to check if we should flush the websocket (So if we should send all messages in queue)
    flush_checker: FlushChecker
}

impl MessagingClient {
    /// Connects to the given server and returns the newly constructed client
    pub async fn new(onion_hostname: &str) -> Result<Self> {
        // Sending the status update to the frontend (So the user knows what's going on)
        let _ = get_app()
            .await
            .emit_payload(WsClientUpdatePayload {
                hostname: onion_hostname.to_string(),
                status: WsClientStatus::ConnectingProxy,
            })
            .map_err(|e| warn!("[CLIENT] Could not emit ws client update: {:?}", e));

        debug!("[CLIENT] Creating verify packet...");
        // Creating a veriy packet to send to the client
        let verify_packet = C2SPacket::identity(onion_hostname).await?;

        #[cfg(not(feature = "dev"))]
        let connect_host = onion_hostname.to_string();
        #[cfg(feature = "dev")]
        let connect_host = onion_hostname
            .replace("-dev-server", "")
            .replace("-dev-client", "");
        // The address which is used to connect to the websocket
        let onion_addr = format!("ws://{}.onion/ws/", connect_host);

        debug!("[CLIENT] Creating proxy...");
        // Creating the Socks5Proxy client,u sed to connect to the tor network
        let proxy = SocksProxy::new()?;
        debug!("[CLIENT] Connecting Proxy...");
        let mut onion_addr = Url::parse(&onion_addr)?;
        onion_addr
            .set_scheme("ws")
            .or(Err(anyhow!("[CLIENT] Could not set scheme")))?;

        // Connecting to the onion server over the tor network
        let sock = proxy.connect(&onion_addr).await?;

        debug!("[CLIENT] Connecting Tungstenite...");
        // Again, sending a new status update
        let _ = get_app()
            .await
            .emit_payload(WsClientUpdatePayload {
                hostname: onion_hostname.to_string(),
                status: WsClientStatus::ConnectingHost,
            })
            .map_err(|e| warn!("[CLIENT] Could not emit ws client update: {:?}", e));


        // Connecting to the websocket with the client
        let (ws_stream, _) = tokio_tungstenite::client_async(&onion_addr, sock).await?;

        // Splitting the duplex stream into a read and write stream
        let (mut write, read) = ws_stream.split();

        // Sending the status update to the frontend, again
        let _ = get_app()
            .await
            .emit_payload(WsClientUpdatePayload {
                hostname: onion_hostname.to_string(),
                status: WsClientStatus::WaitingIdentity,
            })
            .map_err(|e| warn!("[CLIENT] Could not emit ws client update: {:?}", e));

        debug!("[CLIENT] Sending verify packet");
        // And actually sending the verify packet
        write.send(verify_packet.try_into()?).await?;

        // Establishing the channel used to communicate with the common messaging manager
        let (tx, rx) = async_channel::unbounded();

        let arc_write = Arc::new(Mutex::new(write));

        // Spawning the new flush thread
        let checker = FlushChecker::new(arc_write.clone()).await?;
        let flusher_exit = checker.should_exit.clone();

        // Constructing the client as the connection was successful
        let mut c = Self {
            write: arc_write.clone(),
            heartbeat_thread: Arc::new(None),
            receiver: onion_hostname.to_string(),

            rx,
            read_thread: Arc::new(None),
            flush_checker: checker
        };

        debug!("[CLIENT] Spawning heartbeat thread");
        // Spawning the heartbeat thread
        c.spawn_heartbeat_thread();

        // Spawning the read thread
        c.spawn_read_thread(tx, read, arc_write, flusher_exit);

        return Ok(c);
    }

    /// Adds the given packet to the send queue of the client
    pub async fn feed_packet(&self, msg: C2SPacket) -> Result<()> {
        debug!("[CLIENT] Locking write mutex...");
        let mut state = self.write.lock().await;
        debug!("[CLIENT] Feeding packet {:?}...", name_struct!(msg));
        state.feed(msg.try_into()?).await?;
        self.flush_checker.mark_dirty().await;
        debug!("[CLIENT] Done feeding packet.");

        Ok(())
    }

    /// Spawns a thread to read incoming packets from the server.
    /// Handles other misc packages and sends messages to the common messaging manager
    fn spawn_read_thread(
        &mut self,
        tx: Sender<S2CPacket>,
        receiver: ReadStream,
        write: Arc<Mutex<WriteStream>>,
        flush_exit: Arc<AtomicBool>
    ) {
        // Don't spawn a new thread if one already exists
        if self.read_thread.is_some() {
            warn!("[CLIENT] Could not thread read thread, already exists ({:?})", self);
            return;
        }

        let tmp = self.receiver.clone();
        let handle = thread::spawn(move || {
            // Handling the incoming packets concurrently (more performance)
            let future = receiver.for_each_concurrent(2, |msg| {
                let receiver = tmp.clone();
                let write = write.clone();
                let tx = tx.clone();

                // Whole async block is needed to be able to use async/await
                async move {
                    if msg.is_err() {
                        warn!("[CLIENT] Could not parse client {:?}", msg.unwrap_err());
                        return;
                    }

                    let msg = msg.unwrap();
                    if msg.is_pong() {
                        return;
                    }

                    // We are only interested in binary messages, so it can be parsed to a struct
                    if !msg.is_binary() {
                        debug!("[CLIENT] Received non binary message, returning");
                        return;
                    }

                    //  Converting the message to a binary vector
                    let bin = msg.into_data();
                    let packet = S2CPacket::try_from(&bin);
                    if let Err(e) = packet {
                        warn!("[CLIENT] Could not parse packet {:?}", e);
                        return;
                    }

                    // The deserialized packet
                    let packet = packet.unwrap();
                    // And handle the packet
                    let res = Self::handle_packet(packet, &receiver, write, tx).await;
                    if let Err(e) = res {
                        warn!("[CLIENT] Could not handle packet: {:?}", e);
                        return;
                    }
                }
            });

            // Waiting for the stream to end
            block_on(future);

            // And closing everything in this thread
            info!("[CLIENT] Client disconnected for {}", tmp);

            flush_exit.store(true, Ordering::Relaxed);
            let f = block_on(MESSAGING.read());
            block_on(f.remove_connection(&tmp));
        });

        self.read_thread = Arc::new(Some(handle));
    }

    /// Handles the given packet with environment variables (such as receiver, write stream, etc.)
    async fn handle_packet(
        packet: S2CPacket,
        receiver: &str,
        write: Arc<Mutex<WriteStream>>,
        tx: Sender<S2CPacket>,
    ) -> Result<()> {
        // If is Some, we have a packet to which verification is needed
        let mut process_further = None;
        match packet {
            // Don't need that packet for now, it's a placeholder
            S2CPacket::DisconnectMultipleConnections => todo!(),
            // The server sent us a message to verify its identity, so we check and send the IdentityVerified packet back
            S2CPacket::VerifyIdentity(identity) => {
                info!("[CLIENT] Verifying identity for {:?}...", identity);
                // Verifying the identity
                identity.verify().await?;

                debug!("[CLIENT] Identity verified! Locking messaging...");

                // Setting the verify status to true in the messaging manager
                let mgr = MESSAGING.read().await;
                mgr.set_remote_verified(receiver).await?;

                // And possibly sending the IdentityVerified packet
                mgr.check_verified(receiver).await?;

                debug!("[CLIENT] Sending IdentityVerified packet...");
                write
                    .lock()
                    .await
                    .send(C2SPacket::IdentityVerified.try_into()?)
                    .await?;

                debug!("[CLIENT] Done sending IdentityVerified packet.")
            }
            S2CPacket::IdentityVerified => {
                info!("[CLIENT] Got myself verified!");

                // Sending the status update to the frontend
                let mgr = MESSAGING.read().await;
                mgr.set_self_verified(receiver).await?;
                mgr.check_verified(receiver).await?;
            }
            p => process_further = Some(p),
        }

        if process_further.is_none() {
            return Ok(());
        }

        let process_further = process_further.unwrap();
        MESSAGING.read().await.assert_verified(receiver).await?;

        match process_further {
            S2CPacket::Message(msg) => {
                // Redirecting msg to main handler
                tx.send(S2CPacket::Message(msg)).await?;
            }
            // The server sent us a message status update, so we set the status of the message in the messaging manager
            S2CPacket::MessageReceived(date) => {
                MESSAGING
                    .read()
                    .await
                    .set_msg_status(receiver, date, WsMessageStatus::Success)
                    .await?;
            }
            // Same as above but with a failed packet this time
            S2CPacket::MessageFailed(date) => {
                debug!("[CLIENT] Received Server Packet, setting failed");
                MESSAGING
                    .read()
                    .await
                    .set_msg_status(receiver, date, WsMessageStatus::Failed)
                    .await?;
            },
            _ => error!("[CLIENT] Could not process packet {:?}", process_further)
        }

        Ok(())
    }
}

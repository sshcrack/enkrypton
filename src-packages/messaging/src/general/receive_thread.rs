use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use actix_web::Either;
use anyhow::{anyhow, Result};
use encryption::rsa_decrypt;
use log::{debug, error, warn};
use payloads::{
    event::AppHandleExt,
    packets::{C2SPacket, S2CPacket},
    payloads::{WsMessagePayload, WsMessageStatus, WsClientUpdatePayload, WsClientStatus},
};
use shared::get_app;
use smol::block_on;
use storage_internal::{helpers::ChatStorageHelper, STORAGE};
use tokio::sync::RwLock;

use super::{ConnInfo, Connection, MESSAGING};

/// A thread which reads messages incoming from the specific handlers such as `ws_manager` and `MessagingClient`
#[derive(Debug)]
pub struct ConnectionReadThread {
    /// The thread handle that was spawned
    pub read_thread: JoinHandle<()>,
}

impl ConnectionReadThread {
    pub async fn new(conn: &Connection) -> Self {
        let handle = Self::spawn(conn).await;
        Self {
            read_thread: handle,
        }
    }

    pub async fn handle_inner(date: u128, msg: Vec<u8>, receiver_host: &str) -> Result<String> {
        debug!("Reading conn for {}...", receiver_host);
        let storage = block_on(STORAGE.read());

        let priv_key = storage.get_data(|e| {
            e.chats
                .get(receiver_host)
                .and_then(|e| Some(e.priv_key.clone()))
                .ok_or(anyhow!("The private key was empty (should never happen)"))
        });

        debug!("Getting res future");

        let priv_key = block_on(priv_key)?;
        debug!("Done");
        drop(storage);

        let msg = rsa_decrypt(msg, priv_key)?;
        let msg = String::from_utf8(msg)?;

        println!(
            "Received message: {}, Sending payload with receiver {}",
            msg, receiver_host
        );

        STORAGE
            .read()
            .await
            .add_msg(&receiver_host, false, &msg, date)
            .await?;

        Ok(msg)
    }

    /// Handle an incoming message, decrypt and store it
    async fn handle(
        msg: Option<(u128, Vec<u8>)>,
        info: Arc<RwLock<ConnInfo>>,
        receiver_host: &str,
    ) -> Result<()> {
        if msg.is_none() {
            debug!("Skipping...");
            return Ok(());
        }

        // We have to be verified
        MESSAGING
            .read()
            .await
            .assert_verified(&receiver_host)
            .await?;

        let (date, msg) = msg.unwrap();
        // Just a wrapper around handling the message to catch errors
        let res = Self::handle_inner(date, msg, receiver_host).await;
        if let Ok(msg) = res.as_ref() {
            // Setting the status to success and sending a received status to the other side
            MESSAGING
                .read()
                .await
                .set_msg_status(&receiver_host, date, WsMessageStatus::Success)
                .await?;

            match &*info.read().await {
                ConnInfo::Client(c) => {
                    debug!("Client msg");
                    let packet = C2SPacket::MessageReceived(date);
                    c.feed_packet(packet).await?;
                }
                ConnInfo::Server((_, s)) => {
                    debug!("Server msg");
                    let packet = S2CPacket::MessageReceived(date);

                    s.send(packet).await?;
                }
            };

            let handle = get_app().await;
            handle.emit_payload(WsMessagePayload {
                receiver: receiver_host.to_string(),
                message: msg.to_string(),
            })?;
        }

        if let Err(e) = res {
            // Setting the status to failed and sending a failed status to the other side

            error!("Could not handle message: {:?}", e);
            MESSAGING
                .read()
                .await
                .set_msg_status(&receiver_host, date, WsMessageStatus::Failed)
                .await?;

            match &*info.read().await {
                ConnInfo::Client(c) => {
                    debug!("Client msg");
                    let packet = C2SPacket::MessageFailed(date);
                    c.feed_packet(packet).await?;
                }
                ConnInfo::Server((_, s)) => {
                    debug!("Server msg");
                    let packet = S2CPacket::MessageFailed(date);

                    s.send(packet).await?;
                }
            };
        }

        Ok(())
    }

    /// Spawn the read thread with JoinHandle
    pub async fn spawn(conn: &Connection) -> JoinHandle<()> {
        let info_read = conn.info.clone();
        let receiver = info_read.read().await.get_receiver();

        let receiver_host = conn.receiver_host.clone();
        // Spawns the new read thrad
        thread::Builder::new()
            .name(format!("conn-reader-{}", receiver_host))
            .spawn(move || {
                loop {
                    let msg = match &receiver {
                        Either::Left(r) => {
                            // Reads the message from the receiver
                            let msg = block_on(r.recv());
                            if let Err(e) = msg {
                                error!("Could not read packet: {:?}", e);
                                break;
                            }

                            // Handle the message and store it
                            match msg.unwrap() {
                                S2CPacket::Message(msg) => Some(msg),
                                _ => {
                                    warn!("Main Manager received message it could not handle");
                                    None
                                }
                            }
                        }

                        Either::Right(r) => {
                            // Reads the message from the receiver
                            let msg = block_on(r.recv());
                            if let Err(e) = msg {
                                error!("Could not read packet: {:?}", e);
                                break;
                            }

                            // Handle the message and store it
                            match msg.unwrap() {
                                C2SPacket::Message(msg) => Some(msg),
                                _ => {
                                    warn!("Main Manager received message it could not handle");
                                    None
                                }
                            }
                        }
                    };

                    // Handles the message
                    let h = Self::handle(msg, info_read.clone(), &receiver_host);
                    // And wait for it to be handled
                    //TODO do not block here
                    let h = block_on(h);

                    if let Err(e) = h {
                        error!("{:?}", e);
                    }
                }
                let msg = MESSAGING.read();
                let msg = block_on(msg);
                let f = msg.remove_connection(&receiver_host);
                block_on(f);

                let _ = block_on(get_app())
                .emit_payload(WsClientUpdatePayload {
                    hostname: receiver_host.to_string(),
                    status: WsClientStatus::Disconnected,
                })
                .map_err(|e| warn!("[SERVER] Could not emit ws client update: {:?}", e));

            })
            .unwrap()
    }
}

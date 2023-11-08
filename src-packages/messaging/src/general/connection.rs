use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{client::MessagingClient, server::ws_manager::ServerChannels};
#[cfg(feature = "dev")]
use crate::tor::service::get_service_hostname;

use actix_web::Either;
use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use encryption::{rsa_decrypt, rsa_encrypt};
use log::{debug, error, info, warn};
use payloads::{payloads::{WsClientUpdate, WsClientStatus, WsMessagePayload}, packets::{S2CPacket, C2SPacket}};
use shared::{APP_HANDLE, get_app};
use smol::block_on;
use storage_internal::{STORAGE, helpers::ChatStorageHelper};
use tauri::Manager;
use tokio::sync::RwLock;

#[derive(Debug)]
enum ConnInfo {
    Client(MessagingClient),
    Server(ServerChannels),
}

impl ConnInfo {
    pub fn get_receiver(&self) -> Either<Receiver<S2CPacket>, Receiver<C2SPacket>> {
        match self {
            ConnInfo::Client(c) => Either::Left(c.rx.clone()),
            ConnInfo::Server((rx, _)) => Either::Right(rx.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    info: Arc<RwLock<ConnInfo>>,
    read_thread: Arc<Option<JoinHandle<()>>>,
    pub(crate) self_verified: Arc<RwLock<bool>>,
    pub(crate) verified: Arc<RwLock<bool>>,
    receiver_host: String,

    notifier_ready_tx: Sender<()>,
    notifier_ready_rx: Receiver<()>,
}

impl Connection {
    pub(super) async fn notify_verified(&self) -> Result<()> {
        info!(
            "Verified ourselves for connection {:?}!",
            self.receiver_host
        );

        let res = APP_HANDLE
            .read()
            .await
            .as_ref()
            .ok_or(anyhow!(
                "Could not send client update, app handle not there"
            ))
            .and_then(|e| {
                block_on(block_on(STORAGE.read()).get_data(|e| {
                    println!("Current chats are: {:?}", e.chats);
                    Ok(())
                })).unwrap();
                e.emit_all(
                    "ws_client_update",
                    WsClientUpdate {
                        hostname: self.receiver_host.clone(),
                        status: WsClientStatus::CONNECTED,
                    },
                )?;
                Ok(())
            });

        if let Err(e) = res {
            error!("Could not send client update: {:?}", e);
        }

        self.notifier_ready_tx.send(()).await?;
        Ok(())
    }

    pub async fn wait_until_verified(&self) -> Result<()> {
        if *self.self_verified.read().await && *self.verified.read().await {
            return Ok(());
        }

        self.notifier_ready_rx.recv().await?;
        Ok(())
    }

    pub async fn new_client(receiver_host: &str, c: MessagingClient) -> Self {
        let (tx, rx) = async_channel::unbounded();

        let mut s = Self {
            info: Arc::new(RwLock::new(ConnInfo::Client(c))),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            receiver_host: receiver_host.to_string(),

            notifier_ready_tx: tx,
            notifier_ready_rx: rx,
        };

        s.spawn_read_thread().await;

        s
    }

    /// This function assumes the identity has already been verified
    pub async fn new_server(receiver_host: &str, c: ServerChannels) -> Self {
        let (tx, rx) = async_channel::unbounded();

        let mut s = Self {
            info: Arc::new(RwLock::new(ConnInfo::Server(c))),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            receiver_host: receiver_host.to_string(),

            notifier_ready_tx: tx,
            notifier_ready_rx: rx,
        };

        s.spawn_read_thread().await;

        s
    }

    pub async fn spawn_read_thread(&mut self) {
        if self.read_thread.is_some() {
            warn!("Could not spawn read thread, already exists ({:?})", self);
            return;
        }

        let receiver = self.info.read().await.get_receiver();
        let verified = self.verified.clone();
        let self_verified = self.self_verified.clone();

        #[cfg(not(feature = "dev"))]
        let receiver_host: String = self.receiver_host.clone();

        #[cfg(feature = "dev")]
        let receiver_host = get_service_hostname(false).await.unwrap().unwrap();

        let handle = thread::spawn(move || {
            loop {
                let msg = match &receiver {
                    Either::Left(r) => {
                        let msg = block_on(r.recv());
                        if let Err(e) = msg {
                            error!("Could not read packet: {:?}", e);
                            break;
                        }

                        match msg.unwrap() {
                            S2CPacket::Message(msg) => Some(msg),
                            _ => {
                                warn!("Main Manager received message it could not handle");
                                None
                            }
                        }
                    }

                    Either::Right(r) => {
                        let msg = block_on(r.recv());
                        if let Err(e) = msg {
                            error!("Could not read packet: {:?}", e);
                            break;
                        }

                        match msg.unwrap() {
                            C2SPacket::Message(msg) => Some(msg),
                            _ => {
                                warn!("Main Manager received message it could not handle");
                                None
                            }
                        }
                    }
                };

                if msg.is_none() {
                    debug!("Skipping...");
                    continue;
                }

                let is_valid = *block_on(verified.read()) && *block_on(self_verified.read());
                if !is_valid {
                    error!("Connection is not verified yet, skipping message");
                    continue;
                }

                let msg = msg.unwrap();
                debug!("Reading conn for {}...", receiver_host);
                let storage = block_on(STORAGE.read());

                let tmp = receiver_host.clone();
                let priv_key = storage.get_data(|e| {
                    e.chats
                        .get(&tmp)
                        .and_then(|e| Some(e.priv_key.clone()))
                        .ok_or(anyhow!("The private key was empty (should never happen)"))
                });

                debug!("Getting res future");

                let priv_key = block_on(priv_key);
                debug!("Done");
                drop(storage);
                if priv_key.is_err() {
                    error!("Could not get private key: {:?}", priv_key.unwrap_err());
                    continue;
                }

                let priv_key = priv_key.unwrap();
                let msg = rsa_decrypt(msg, priv_key);
                if msg.is_err() {
                    error!("Could not decrypt message: {:?}", msg.unwrap_err());
                    continue;
                }

                let msg = String::from_utf8(msg.unwrap());
                if let Err(e) = msg {
                    error!("Could not parse message: {:?}", e);
                    continue;
                }

                let msg = msg.unwrap();
                let event_name = &format!("msg-{}", receiver_host);
                println!(
                    "Received message: {}, Sending payload to {}",
                    msg, event_name
                );

                let res = block_on(
                    block_on(STORAGE.read()) // ..
                        .add_msg(&receiver_host, false, &msg), //..
                );

                if res.is_err() {
                    error!("Could not modify storage data: {:?}", res.unwrap_err());
                }

                let handle = block_on(get_app());
                let res = handle.emit_all(event_name, WsMessagePayload { message: msg });
                if res.is_err() {
                    error!("Could not emit message: {:?}", res.unwrap_err());
                    return;
                }
            }
        });

        self.read_thread = Arc::new(Some(handle));
    }

    pub async fn send_msg(&self, msg: &str) -> Result<()> {
        let raw = msg.as_bytes().to_vec();

        let tmp = self.receiver_host.clone();
        println!("Reading pubkey...");
        let pub_key = STORAGE
            .read()
            .await
            .get_data(|e| {
                e.chats
                    .get(&tmp)
                    .and_then(|e| e.pub_key.clone())
                    .ok_or(anyhow!("The pub key was empty (should never happen)"))
            })
            .await?;

        println!("Sending");
        let bin = rsa_encrypt(raw, &pub_key)?;

        // encrypt here
        match &*self.info.read().await {
            ConnInfo::Client(c) => {
                let packet = C2SPacket::Message(bin);
                c.send_packet(packet).await
            }
            ConnInfo::Server((_, s)) => {
                let packet = S2CPacket::Message(bin);

                s.send(packet).await?;
                Ok(())
            }
        }?;

        // Adding if successful
        STORAGE
            .read()
            .await
            .add_msg(&self.receiver_host, true, msg)
            .await?;

        Ok(())
    }
}

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{
    encryption::{rsa_decrypt, rsa_encrypt}, messaging::payloads::WsMessagePayload, storage::STORAGE, util::get_app,
};

use super::{
    client::MessagingClient,
    packets::{C2SPacket, S2CPacket},
    webserver::ws_manager::ServerChannels,
};
use actix_web::Either;
use anyhow::{anyhow, Result};
use async_channel::Receiver;
use log::{debug, error, warn};
use smol::block_on;
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
    pub(super) self_verified: Arc<RwLock<bool>>,
    pub(super) verified: Arc<RwLock<bool>>,
    host: String,
}

impl Connection {
    pub async fn new_client(host: &str, c: MessagingClient) -> Self {
        let mut s = Self {
            info: Arc::new(RwLock::new(ConnInfo::Client(c))),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            host: host.to_string(),
        };

        s.spawn_read_thread().await;

        s
    }

    /// This function assumes the identity has already been verified
    pub async fn new_server(host: &str, c: ServerChannels) -> Self {
        let mut s = Self {
            info: Arc::new(RwLock::new(ConnInfo::Server(c))),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            host: host.to_string(),
        };

        s.spawn_read_thread().await;

        s
    }

    pub async fn spawn_read_thread(&mut self) {
        if self.read_thread.is_some() {
            warn!("Could not spawn read thread, already exists");
            return;
        }

        let receiver = self.info.read().await.get_receiver();
        let verified = self.verified.clone();
        let self_verified = self.self_verified.clone();

        let host: String = self.host.clone();
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
                let storage = block_on(STORAGE.read());

                let tmp = host.clone();
                let priv_key = storage.get_data(|e| {
                    e.chats
                        .get(&tmp)
                        .and_then(|e| Some(e.priv_key.clone()))
                        .ok_or(anyhow!("The private key was empty (should never happen)"))
                });

                let priv_key = block_on(priv_key);
                if priv_key.is_err() {
                    error!("Could not get private key: {:?}", priv_key.unwrap_err());
                    continue;
                }

                let priv_key = priv_key.unwrap();
                let msg = rsa_decrypt(msg, priv_key);
                let msg = String::from_utf8(msg);
                if let Err(e) = msg {
                    error!("Could not parse message: {:?}", e);
                    continue;
                }

                let msg = msg.unwrap();
                println!("Received message: {}", msg);

                let handle = block_on(get_app());
                let res =
                    handle.emit_all(&format!("msg-{}", host), WsMessagePayload { message: msg });
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

        let tmp = self.host.clone();
        let pub_key = STORAGE.read().await.get_data(|e| {
            e.chats
                .get(&tmp)
                .and_then(|e| e.pub_key.clone())
                .ok_or(anyhow!("The pub key was empty (should never happen)"))
        }).await?;

        let bin = rsa_encrypt(raw, &pub_key);

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
        }
    }
}

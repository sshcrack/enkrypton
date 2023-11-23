use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::{client::MessagingClient, server::ws_manager::ServerChannels};

use actix_web::Either;
use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use encryption::rsa_encrypt;
use log::{debug, error, info};
use payloads::{
    event::AppHandleExt,
    packets::{C2SPacket, S2CPacket},
    payloads::{WsClientStatus, WsClientUpdatePayload, WsMessageStatus},
};
use shared::{APP_HANDLE, util::now_millis};
use smol::block_on;
use storage_internal::{helpers::ChatStorageHelper, STORAGE};
use tokio::sync::RwLock;

use super::{MESSAGING, ConnectionReadThread};

#[derive(Debug)]
pub(super) enum ConnInfo {
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
    pub(super) info: Arc<RwLock<ConnInfo>>,
    read_thread: Arc<Option<ConnectionReadThread>>,
    pub(crate) self_verified: Arc<RwLock<bool>>,
    pub(crate) verified: Arc<RwLock<bool>>,
    pub(super) receiver_host: String,

    already_notified: Arc<AtomicBool>,

    notifier_ready_tx: Sender<()>,
    notifier_ready_rx: Receiver<()>,
}

impl Connection {
    pub(super) async fn notify_verified(&self) -> Result<()> {
        if self.already_notified.load(Ordering::Relaxed) {
            debug!("[CONNECTION] Not notifying again.");
            return Ok(())
        }

        self.already_notified.store(true, Ordering::Relaxed);
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
            .and_then(|handle| {
                block_on(block_on(STORAGE.read()).get_data(|e| {
                    println!("Current chats are: {:?}", e.chats);
                    Ok(())
                }))
                .unwrap();
                handle.emit_payload(WsClientUpdatePayload {
                    hostname: self.receiver_host.clone(),
                    status: WsClientStatus::Connected,
                })?;
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
        println!("New client connection: {:?}", receiver_host);
        let (tx, rx) = async_channel::unbounded();

        let mut s = Self {
            info: Arc::new(RwLock::new(ConnInfo::Client(c))),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            receiver_host: receiver_host.to_string(),
            already_notified: Arc::new(AtomicBool::new(false)),

            notifier_ready_tx: tx,
            notifier_ready_rx: rx,
        };

        let read_thread = ConnectionReadThread::new(&s).await;
        s.read_thread = Arc::new(Some(read_thread));
        s
    }

    /// This function assumes the identity has already been verified
    pub async fn new_server(receiver_host: &str, c: ServerChannels) -> Self {
        println!("New server connection: {:?}", receiver_host);
        let (tx, rx) = async_channel::unbounded();

        let mut s = Self {
            info: Arc::new(RwLock::new(ConnInfo::Server(c))),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            receiver_host: receiver_host.to_string(),
            already_notified: Arc::new(AtomicBool::new(false)),

            notifier_ready_tx: tx,
            notifier_ready_rx: rx,
        };

        let read_thread = ConnectionReadThread::new(&s).await;
        s.read_thread = Arc::new(Some(read_thread));
        s
    }

    pub async fn send_msg(&self, msg: &str) -> Result<()> {
        // Adding if successful
        let date = STORAGE
            .read()
            .await
            .add_msg(&self.receiver_host, true, msg, now_millis())
            .await?;

            debug!("Info of msg is {}", date);
        let res = self.inner_send(msg, date).await;
        if res.is_err() {
            debug!("Inner Send failed, setting status to failed");
            MESSAGING
                .read()
                .await
                .set_msg_status(&self.receiver_host, date, WsMessageStatus::Failed)
                .await?;
        } else {
            debug!("Sending status sent for msg {}", date);
            MESSAGING
                .read()
                .await
                .set_msg_status(&self.receiver_host, date, WsMessageStatus::Sent)
                .await?;
        }

        res?;
        Ok(())
    }

    async fn inner_send(&self, msg: &str, date: u128) -> Result<()> {
        let raw = msg.as_bytes().to_vec();

        let tmp = self.receiver_host.clone();
        println!("Reading pubkey for {}...", tmp);
        let pub_key = STORAGE
            .read()
            .await
            .get_data(|e| {
                e.chats
                    .get(&tmp)
                    .and_then(|e| e.rec_pub_key.clone())
                    .ok_or(anyhow!("The pub key was empty (should never happen)"))
            })
            .await?;

        println!("Sending");
        let bin = rsa_encrypt(raw, &pub_key)?;

        // encrypt here
        match &*self.info.read().await {
            ConnInfo::Client(c) => {
                debug!("Client msg");
                let packet = C2SPacket::Message((date, bin));
                c.feed_packet(packet).await?;
            }
            ConnInfo::Server((_, s)) => {
                debug!("Server msg");
                let packet = S2CPacket::Message((date, bin));

                s.send(packet).await?;
            }
        };

        Ok(())
    }
}

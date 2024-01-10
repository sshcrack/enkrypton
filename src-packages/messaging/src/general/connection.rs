use std::sync::Arc;

use crate::{client::MessagingClient, server::ws_manager::ServerChannels};

use actix_web::Either;
use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
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

/// This enum is used to store the connection info either from the client or the server
#[derive(Debug)]
pub(super) enum ConnInfo {
    Client(MessagingClient),
    Server(ServerChannels),
}

impl ConnInfo {
    /// # Returns
    /// 
    /// Gets the general receiver for the connection (mostly used for messages)
    pub fn get_receiver(&self) -> Either<Receiver<S2CPacket>, Receiver<C2SPacket>> {
        match self {
            ConnInfo::Client(c) => Either::Left(c.rx.clone()),
            ConnInfo::Server((rx, _)) => Either::Right(rx.clone()),
        }
    }
}

/// A generalized connection struct that can be used for both the client and the server
#[derive(Debug, Clone)]
pub struct Connection {
    pub(super) info: Arc<RwLock<ConnInfo>>,
    read_thread: Arc<Option<ConnectionReadThread>>,
    pub(crate) self_verified: Arc<RwLock<bool>>,
    pub(crate) verified: Arc<RwLock<bool>>,
    pub(super) receiver_host: String,

    notifier_ready_tx: Sender<()>,
    notifier_ready_rx: Receiver<()>,
}

impl Connection {
    /// Notifies the frontend about the verified connection
    pub(super) async fn notify_verified(&self) -> Result<()> {
        info!(
            "Verified ourselves for connection {:?}!",
            self.receiver_host
        );

        // Sending the client update
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

        // Notifying other backend listeners (used for wait_until_verified)
        self.notifier_ready_tx.send(()).await?;
        Ok(())
    }

    /// Waits until the connection is verified
    pub async fn wait_until_verified(&self) -> Result<()> {
        // If we are already verified, we can just return
        if *self.self_verified.read().await && *self.verified.read().await {
            return Ok(());
        }

        // Waiting for the notifier to be ready
        self.notifier_ready_rx.recv().await?;
        Ok(())
    }

    /// Creates a new generic connection
    ///
    /// # Arguments
    ///
    /// * `receiver_host` - The receiver host of this connection
    /// * `info` - Additional information and communication channels between structs
    ///
    /// # Returns
    ///
    /// The newly constructed connection
    async fn new_general(receiver_host: &str, info: ConnInfo) -> Self {
        println!("New client connection: {:?}", receiver_host);
        let (tx, rx) = async_channel::unbounded();

        let mut s = Self {
            info: Arc::new(RwLock::new(info)),
            read_thread: Arc::new(None),
            verified: Arc::new(RwLock::new(false)),
            self_verified: Arc::new(RwLock::new(false)),
            receiver_host: receiver_host.to_string(),

            notifier_ready_tx: tx,
            notifier_ready_rx: rx,
        };

        let read_thread = ConnectionReadThread::new(&s).await;
        s.read_thread = Arc::new(Some(read_thread));
        s
    }

    /// Creates a new client connection
    ///
    /// # Arguments
    ///
    /// * `receiver_host` - The receiver host of this connection
    /// * `c` - The messaging client to use for this connection
    ///
    /// # Returns
    ///
    /// The newly constructed connection
    pub async fn new_client(receiver_host: &str, c: MessagingClient) -> Self {
        println!("New client connection: {:?}", receiver_host);
        Self::new_general(receiver_host, ConnInfo::Client(c)).await
    }

    /// Creates a new server connection.
    /// This function assumes that the connection is already verified
    ///
    /// # Arguments
    ///
    /// * `receiver_host` - The receiver host of this connection
    /// * `c` - The channels used to send commands to the server ws handler and to receive messages from the server
    ///
    /// # Returns
    ///
    /// The newly constructed connection
    pub async fn new_server(receiver_host: &str, c: ServerChannels) -> Self {
        println!("New server connection: {:?}", receiver_host);
        Self::new_general(receiver_host, ConnInfo::Server(c)).await
    }

    /// Sends a message to the receiver
    ///
    /// # Arguments
    ///
    /// * `msg` - The message that should be sent
    ///
    /// # Returns
    ///
    /// Whether the message was sent successfully
    pub async fn send_msg(&self, msg: &str) -> Result<()> {
        // Adding message to storage
        let date = STORAGE
            .read()
            .await
            .add_msg(&self.receiver_host, true, msg, now_millis())
            .await?;

            debug!("Info of msg is {}", date);
        // Sending message and setting status later
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

    /// Sends a message to the receiver with the given date and msg, internal function
    async fn inner_send(&self, msg: &str, date: u128) -> Result<()> {
        let raw = msg.as_bytes().to_vec();

        let tmp = self.receiver_host.clone();
        println!("Reading public key for {}...", tmp);

        // Firstly we need to get the public key of the receiver
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
        // And encrypt the message
        let bin = pub_key.encrypt(raw)?;

        // And send it to the receiver
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

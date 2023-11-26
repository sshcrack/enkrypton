use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{info, debug};
use payloads::{
    event::AppHandleExt,
    payloads::{WsMessageStatus, WsMessageStatusPayload},
};
use shared::get_app;
use storage_internal::STORAGE;
use tokio::sync::RwLock;

use crate::client::MessagingClient;

use super::Connection;

/// A Manager which holds the connections by receiver name
pub struct MessagingManager {
    /// The connections by receiver name
    pub(crate) connections: Arc<RwLock<HashMap<String, Connection>>>,
}

lazy_static! {
    /// The global messaging manager
    pub static ref MESSAGING: Arc<RwLock<MessagingManager>> =
        Arc::new(RwLock::new(MessagingManager::new()));
    // The interval for the heartbeat
    pub static ref HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(25);
    pub static ref HEARTBEAT: Duration = HEARTBEAT_TIMEOUT.div_f32(2.0);
}

impl MessagingManager {
    fn new() -> Self {
        MessagingManager {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connects to a receiver and adds it to the connections
    async fn connect(&self, onion_hostname: &str) -> Result<()> {
        let client = MessagingClient::new(&onion_hostname).await?;

        info!("[CLIENT]: New Connection for {}", onion_hostname);

        let conn = Connection::new_client(onion_hostname, client).await;
        self.connections
            .write()
            .await
            .insert(onion_hostname.to_string(), conn);

        Ok(())
    }

    /// Gets a connection by receiver name or connects if it doesn't exist yet
    pub async fn get_or_connect(&self, onion_host: &str) -> Result<Connection> {
        if !self.connections.read().await.contains_key(onion_host) {
            self.connect(&onion_host).await?;
        }

        let res = self.connections.read().await.get(onion_host).cloned();
        if let Some(c) = res {
            return Ok(c.clone());
        }

        return Err(anyhow!("Could not establish connection"));
    }

    /// Waits until a connection with the given name is verified
    pub async fn wait_until_verified(&self, onion_host: &str) -> Result<()> {
        if !self.connections.read().await.contains_key(onion_host) {
            return Err(anyhow!("Connection does not exist"));
        }

        // Gets the connection by onion_host
        let conn = self
            .connections
            .read()
            .await
            .get(onion_host)
            .cloned()
            .unwrap();

        conn.wait_until_verified().await?;
        Ok(())
    }

    /// Checks if the connection with the given host name is verified and sends a notification if newly verified
    pub(crate) async fn check_verified(&self, onion_host: &str) -> Result<()> {
        debug!("Checking verified for {}", onion_host);
        let verified = self.assert_verified(onion_host).await;

        if let Ok(c) = verified {
            c.notify_verified().await?;
        }

        Ok(())
    }

    /// Asserts that the connection with the given host name is verified
    pub(crate) async fn assert_verified(&self, onion_host: &str) -> Result<Connection> {
        debug!("Checking verified for {}", onion_host);
        let res = self
            .connections
            .read()
            .await
            .get(onion_host)
            .cloned()
            .ok_or(anyhow!(
                "assert_verified should only be callable after a connection is established"
            ))?;

        let remote_verified = *res.verified.read().await;
        let self_verified = *res.self_verified.read().await;
        if !remote_verified {
            return Err(anyhow!("Remote is not verified yet"))
        }

        if !self_verified {
            return Err(anyhow!("Remote is not verified yet"))
        }

        Ok(res)
    }

    /// Checks if we are connected to the given host
    pub async fn is_connected(&self, onion_host: &str) -> bool {
        self.connections.read().await.contains_key(onion_host)
    }

    /// Removes the connection with the given host name
    pub async fn remove_connection(&self, onion_host: &str) {
        self.connections.write().await.remove(onion_host);
    }

    /// Setting the new msg status and updates the storage/frontend
    pub async fn set_msg_status(
        &self,
        onion_host: &str,
        date: u128,
        status: WsMessageStatus,
    ) -> Result<()> {
        STORAGE
            .read()
            .await
            .modify_storage_data(|d| {
                let chat = d
                    .chats
                    .get_mut(onion_host)
                    .ok_or(anyhow!("Could not find chat"))?;

                let msg = chat
                    .messages
                    .iter_mut()
                    .find(|e| e.date == date)
                    .ok_or(anyhow!(format!("Could not set status: Message with date {} not found with receiver {}", date, onion_host)))?;

                msg.status = status.clone();
                Ok(())
            })
            .await?;

        debug!("Sending message status update to client Hostname: {}, Date: {}, Status: {:?}", onion_host, date, status);
        get_app().await.emit_payload(WsMessageStatusPayload {
            hostname: onion_host.to_string(),
            date,
            status,
        })?;
        Ok(())
    }
}

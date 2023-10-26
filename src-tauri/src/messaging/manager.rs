use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::info;
use tauri::Manager;
use tokio::sync::RwLock;

use crate::{util::get_app, messaging::payloads::{WsClientUpdate, WsClientStatus}};

use super::{client::MessagingClient, Connection};

pub struct MessagingManager {
    pub(super) connections: HashMap<String, Connection>,
}

lazy_static! {
    pub static ref MESSAGING: Arc<RwLock<MessagingManager>> =
        Arc::new(RwLock::new(MessagingManager::new()));
    pub static ref HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(25);
    pub static ref HEARTBEAT: Duration = HEARTBEAT_TIMEOUT.div_f32(2.0);
}

impl MessagingManager {
    fn new() -> Self {
        MessagingManager {
            connections: HashMap::new(),
        }
    }

    async fn connect(&mut self, onion_hostname: &str) -> Result<()> {
        let client = MessagingClient::new(&onion_hostname).await?;

        info!("[CLIENT]: New Connection for {}", onion_hostname);

        let conn = Connection::new_client(onion_hostname, client).await;
        self.connections
            .insert(onion_hostname.to_string(), conn);

        let handle = get_app().await;
        handle.emit_all("ws_client_update", WsClientUpdate {
            hostname: onion_hostname.to_string(),
            status: WsClientStatus::CONNECTED
        })?;
        Ok(())
    }

    pub async fn get_or_connect(&mut self, onion_host: &str) -> Result<Connection> {
        if !self.connections.contains_key(onion_host) {
            self.connect(&onion_host).await?;
        }

        let res = self.connections.get(onion_host);
        if let Some(c) = res {
            return Ok(c.clone());
        }

        return Err(anyhow!("Could not establish connection"));
    }

    pub async fn wait_until_verified(&self, onion_host: &str) -> Result<()> {
        if !self.connections.contains_key(onion_host) {
            return Err(anyhow!("Connection does not exist"));
        }

        let conn = self.connections.get(onion_host).unwrap();
        conn.wait_until_verified().await?;
        Ok(())
    }


    pub(super) async fn check_verify_status(&self, onion_host: &str) -> Result<()> {
        let res = self.connections.get(onion_host)
            .ok_or(anyhow!("check_verify_status should only be callable after a connection is established"))?;

        let remote_verified = *res.verified.read().await;
        let self_verified = *res.self_verified.read().await;

        if remote_verified && self_verified {
            res.notify_verified().await?;
        }

        Ok(())
    }

    pub fn is_connected(&self, onion_host: &str) -> bool {
        self.connections.contains_key(onion_host)
    }

    pub fn remove_connection(&mut self, onion_host: &str) {
        self.connections.remove(onion_host);
    }
}

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::info;
use tokio::sync::RwLock;

use super::{client::MessagingClient, webserver::ws_manager::WsActor, Connection};

pub struct MessagingManager {
    connections: HashMap<String, Connection>,
}

lazy_static! {
    pub static ref MESSAGING: Arc<RwLock<MessagingManager>> =
        Arc::new(RwLock::new(MessagingManager::new()));
    pub static ref HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);
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
        self.connections
            .insert(onion_hostname.to_string(), Connection::new_client(client));

        Ok(())
    }

    pub(super) fn insert_server(&mut self, onion_host: &str, a: &WsActor) {
        info!("[SERVER]: New Connection for {}", onion_host);
        self.connections.insert(
            onion_host.to_string(),
            Connection::new_server((a.c_rx.clone(), a.s_tx.clone())),
        );
    }

    pub async fn get_or_connect(&mut self, onion_host: &str) -> Result<Connection> {
        if !self.connections.contains_key(onion_host) {
            self.connect(&onion_host).await?;
        }

        let res = self.connections.get(onion_host);
        if let Some(c) = res {
            return Ok(c.clone());
        }

        return Err(anyhow!("Could not establish connection"))
    }

    pub async fn get_connection(&self, onion_host: &str) -> Result<Connection> {
        let res = self.connections.get(onion_host);
        if let Some(c) = res {
            return Ok(c.clone());
        }

        return Err(anyhow!("Could not establish connection"))
    }

    pub fn is_connected(&self, onion_host: &str) -> bool {
        self.connections.contains_key(onion_host)
    }

    pub fn remove_connection(&mut self, onion_host: &str) {
        self.connections.remove(onion_host);
    }
}

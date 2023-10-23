use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use super::{client::MessagingClient, webserver::ws_manager::MessagingServer};

pub enum Role {
    Client2Server(MessagingClient),
    Server2Client(MessagingServer)
}

pub struct MessagingManager {
    connections: HashMap<String, Role>
}

lazy_static! {
    pub static ref MESSAGING: Arc<RwLock<MessagingManager>> = Arc::new(RwLock::new(MessagingManager::new()));
}

impl MessagingManager {
    fn new() -> Self {
        MessagingManager {
            connections: HashMap::new()
        }
    }

    pub async fn connect(&mut self, onion_hostname: &str) -> Result<()> {
        let client = MessagingClient::new(&onion_hostname).await?;
        self.connections.insert(onion_hostname.to_string(), Role::Client2Server(client));

        Ok(())
    }

    pub(super) async fn insert_server(&mut self, onion_host: &str, messaging: MessagingServer) -> Result<()> {
        self.connections.insert(onion_host.to_string(), Role::Server2Client(messaging));

        Ok(())
    }

    pub fn remove_link(&mut self, onion_host: &str) {
        self.connections.remove(onion_host);
    }
}
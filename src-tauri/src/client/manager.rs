use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use async_trait::async_trait;

use super::MessagingClient;

#[async_trait]
pub trait ClientManager {
    async fn get_or_create(onion_addr: &str) -> Result<MessagingClient>;
}

#[async_trait]
impl ClientManager for MessagingClient {
    async fn get_or_create(onion_addr: &str) -> Result<MessagingClient> {
        let read = CLIENTS.read().await;
        if let Some(c) = read.get(onion_addr) {
            return Ok(c.clone());
        }

        drop(read);
        let mut write = CLIENTS.write().await;

        let new_c = MessagingClient::new(onion_addr).await?;

        write.insert(onion_addr.to_string(), new_c.clone());

        new_c.create_listen_thread().await?;
        Ok(new_c)
    }
}

lazy_static! {
    static ref CLIENTS: Arc<RwLock<HashMap<String, MessagingClient>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

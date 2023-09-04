use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::util::is_onion_hostname;

use super::MessagingClient;

#[async_trait]
pub trait ClientManager {
    async fn get_or_create(onion_hostname: &str) -> Result<MessagingClient>;
}

#[async_trait]
impl ClientManager for MessagingClient {
    async fn get_or_create(onion_hostname: &str) -> Result<MessagingClient> {
        if !is_onion_hostname(onion_hostname) {
            return Err(anyhow!("Invalid hostname '{}' on get_or_create ClientManager", onion_hostname));
        }

        let read = CLIENTS.read().await;
        if let Some(c) = read.get(onion_hostname) {
            return Ok(c.clone());
        }

        drop(read);
        let mut write = CLIENTS.write().await;

        let new_c = MessagingClient::new(onion_hostname).await?;

        write.insert(onion_hostname.to_string(), new_c.clone());

        new_c.create_listen_thread().await?;
        Ok(new_c)
    }
}

lazy_static! {
    static ref CLIENTS: Arc<RwLock<HashMap<String, MessagingClient>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

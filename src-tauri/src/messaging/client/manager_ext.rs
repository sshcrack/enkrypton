use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::messaging::MessagingManager;

#[async_trait]
pub trait ManagerExt {
    async fn set_remote_verified(&mut self, onion_host: &str) -> Result<()>;
    async fn set_self_verified(&mut self, onion_host: &str) -> Result<()>;
}

#[async_trait]
impl ManagerExt for MessagingManager {
    async fn set_remote_verified(&mut self, onion_host: &str) -> Result<()> {
        let res = self.connections.get_mut(onion_host)
            .ok_or(anyhow!("set_remote_verified should only be callable after a connection is established"))?;

        *res.verified.write().await = true;
        Ok(())
    }

    async fn set_self_verified(&mut self, onion_host: &str) -> Result<()> {
        let res = self.connections.get_mut(onion_host)
            .ok_or(anyhow!("set_remote_verified should only be callable after a connection is established"))?;

        *res.self_verified.write().await = true;
        Ok(())
    }
}
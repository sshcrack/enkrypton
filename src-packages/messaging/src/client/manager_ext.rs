use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::general::MessagingManager;

#[async_trait]
/// Extends the `Manager` struct to have some client helper functions.
pub trait ManagerExt {
    /// Sets the remote verification status for the specified `onion_host`.
    /// This is used to check if both remote and self are verified
    /// to create a safe environment for messages to be sent.
    ///
    /// # Arguments
    ///
    /// * `onion_host` - The onion host to set the verification status for.
    ///
    /// # Returns
    ///
    /// A `Result` whether it was successful or not
    async fn set_remote_verified(&self, onion_host: &str) -> Result<()>;

    /// Same as `set_remote_verified` but for the self verification status.
    /// Again, this is used to check if both remote and self are verified
    /// And to establish a safe environment for messages to be sent.
    ///
    /// # Arguments
    /// 
    /// * `onion_host` - The onion host to set the verification status for.
    ///
    /// # Returns
    ///
    /// The `Result` whether it was successful or not
    async fn set_self_verified(&self, onion_host: &str) -> Result<()>;
}

#[async_trait]
impl ManagerExt for MessagingManager {
    async fn set_remote_verified(&self, onion_host: &str) -> Result<()> {
        // Getting current connection details and infos with the server
        let res = self.connections.read().await.get(onion_host).cloned()
            .ok_or(anyhow!("set_remote_verified should only be callable after a connection is established"))?;

        // Setting the remote verification status to true
        *res.verified.write().await = true;
        Ok(())
    }

    async fn set_self_verified(&self, onion_host: &str) -> Result<()> {
        // Getting the connection of the given id
        let res = self.connections.read().await.get(onion_host).cloned()
            .ok_or(anyhow!("set_remote_verified should only be callable after a connection is established"))?;

        // Setting the self verification status to true
        *res.self_verified.write().await = true;
        Ok(())
    }
}

use async_trait::async_trait;
use log::info;

use crate::general::{Connection, MessagingManager};

use super::ws_manager::WsActor;

/// Extends the manager with some helper functions for the server
#[async_trait]
pub trait ManagerExt {
    /// Sets the remote verified flag for a connection (just a helper function)
    async fn set_remote_verified(&self, onion_host: &str, a: &WsActor);
    /// Sets the self verified flag for a connection (just a helper function)
    async fn set_self_verified(&self, onion_host: &str, a: &WsActor);
    /// Inserts or adds a connection to the manager and returns it
    async fn get_or_insert(&self, onion_host: &str, a: &WsActor) -> Connection;
}

#[async_trait]
impl ManagerExt for MessagingManager {
    async fn set_remote_verified(&self, onion_host: &str, a: &WsActor) {
        let conn = self.get_or_insert(onion_host, a).await;
        *conn.verified.write().await = true;

        info!("[SERVER]: New Connection for {}", onion_host);
    }

    async fn set_self_verified(&self, onion_host: &str, a: &WsActor) {
        let conn = self.get_or_insert(onion_host, a).await;
        *conn.self_verified.write().await = true;
    }

    async fn get_or_insert(&self, onion_host: &str, a: &WsActor) -> Connection {
        let r = self.connections.read().await.get(onion_host).cloned();
        if let Some(r) = r {
            return r;
        }

        // Creates a new connection
        let c = Connection::new_server(onion_host, (*a.c_rx.clone(), *a.s_tx.clone())).await;
        self.connections
            .write()
            .await
            .insert(onion_host.to_string(), c.clone());

        c
    }
}

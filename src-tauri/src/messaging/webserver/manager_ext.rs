use async_trait::async_trait;
use log::info;
use smol::block_on;

use crate::messaging::{MessagingManager, Connection};

use super::ws_manager::WsActor;

#[async_trait]
pub trait ManagerExt {
    async fn set_remote_verified(&self, onion_host: &str, a: &WsActor);
    async fn set_self_verified(&self, onion_host: &str, a: &WsActor);
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

        let c = Connection::new_server(onion_host, (a.c_rx.clone(), a.s_tx.clone())).await;
        let e = self.connections
            .write().await.insert(onion_host.to_string(), c.clone());

        e
    }
}
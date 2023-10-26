use async_trait::async_trait;
use log::info;
use smol::block_on;

use crate::messaging::{MessagingManager, Connection};

use super::ws_manager::WsActor;

#[async_trait]
pub trait ManagerExt {
    async fn set_remote_verified(&mut self, onion_host: &str, a: &WsActor);
    async fn set_self_verified(&mut self, onion_host: &str, a: &WsActor);
    fn get_or_insert(&mut self, onion_host: &str, a: &WsActor) -> &mut Connection;
}

#[async_trait]
impl ManagerExt for MessagingManager {
    async fn set_remote_verified(&mut self, onion_host: &str, a: &WsActor) {
        let conn = self.get_or_insert(onion_host, a);
        *conn.verified.write().await = true;

        info!("[SERVER]: New Connection for {}", onion_host);
    }

    async fn set_self_verified(&mut self, onion_host: &str, a: &WsActor) {
        let conn = self.get_or_insert(onion_host, a);
        *conn.self_verified.write().await = true;
    }

    fn get_or_insert(&mut self, onion_host: &str, a: &WsActor) -> &mut Connection {
        self.connections
            .entry(onion_host.to_string())
            .or_insert_with(||{
                let c = Connection::new_server(onion_host, (a.c_rx.clone(), a.s_tx.clone()));

                //TODO don't block
                block_on(c)
            })
    }
}
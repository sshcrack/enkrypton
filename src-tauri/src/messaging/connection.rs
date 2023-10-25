use std::sync::Arc;

use super::{
    client::MessagingClient,
    packages::{C2SPacket, S2CPacket},
    webserver::ws_manager::ServerChannels,
};
use anyhow::Result;
use tokio::sync::RwLock;

#[derive(Debug)]
enum ConnInfo {
    Client(MessagingClient),
    Server(ServerChannels),
}

#[derive(Debug, Clone)]
pub struct Connection {
    info: Arc<RwLock<ConnInfo>>,
    verified: bool
}

impl Connection {
    pub fn new_client(c: MessagingClient) -> Self {
        Self {
            info: Arc::new(RwLock::new(ConnInfo::Client(c))),
            verified: false
        }
    }

    /// This function assumes the identity has already been verified
    pub fn new_server(c: ServerChannels) -> Self {
        Self {
            info: Arc::new(RwLock::new(ConnInfo::Server(c))),
            verified: true
        }
    }

    pub async fn send_msg(&self, msg: &str) -> Result<()> {
        let bin = msg.as_bytes().to_vec();

        // encrypt here
        match &*self.info.read().await {
            ConnInfo::Client(c) => {
                let packet = C2SPacket::Message(bin);
                c.send_packet(packet).await
            }
            ConnInfo::Server((_, s)) => {
                let packet = S2CPacket::Message(bin);

                s.send(packet).await?;
                Ok(())
            }
        }
    }
}

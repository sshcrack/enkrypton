use anyhow::{Result, anyhow};
use async_trait::async_trait;
use payloads::{data::ChatMessage, payloads::WsMessageStatus};
use shared::util::now_millis;

use crate::StorageManager;

#[async_trait]
pub trait ChatStorageHelper {
    async fn add_msg(&self, receiver: &str, sent_self: bool, msg: &str) -> Result<u128>;
}

#[async_trait]
impl ChatStorageHelper for StorageManager {
    async fn add_msg(&self, receiver: &str, sent_self: bool, msg: &str) -> Result<u128> {
        let date = now_millis();

        let status = if sent_self { WsMessageStatus::Sending } else { WsMessageStatus::Success };

        self.modify_storage_data(|e| {
            let c = e
                .chats
                .get_mut(receiver)
                .ok_or(anyhow!("Chat to add message to could not be found"))?;

            c.messages.push(ChatMessage {
                self_sent: sent_self,
                msg: msg.to_string(),
                date,
                status
            });

            Ok(())
        })
        .await?;

        return Ok(date);
    }
}

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use directories::util::now_millis;

use crate::{StorageManager, ChatMessage};

#[async_trait]
pub trait ChatStorageHelper {
    async fn add_msg(&self, receiver: &str, sent_self: bool, msg: &str) -> Result<()>;
}

#[async_trait]
impl ChatStorageHelper for StorageManager {
    async fn add_msg(&self, receiver: &str, sent_self: bool, msg: &str) -> Result<()> {
        self.modify_storage_data(|e| {
            let c = e
                .chats
                .get_mut(receiver)
                .ok_or(anyhow!("Chat to add message to could not be found"))?;

            c.messages.push(ChatMessage {
                self_sent: sent_self,
                msg: msg.to_string(),
                date: now_millis(),
            });

            Ok(())
        })
        .await
    }
}

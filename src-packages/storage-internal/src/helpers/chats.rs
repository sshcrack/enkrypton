use anyhow::{Result, anyhow};
use async_trait::async_trait;
use payloads::{data::ChatMessage, payloads::{WsMessageStatus, WsMessageStatusPayload}, event::AppHandleExt};
use shared::APP_HANDLE;

use crate::StorageManager;

/// Just an extension trait for the storage manager to add messages to chats
#[async_trait]
/// A trait for chat storage helpers.
pub trait ChatStorageHelper {
    /// Adds a message to the secure chat storage.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The receiver of the message (onion hostname)
    /// * `sent_self` - Whether the message was sent by the user or not
    /// * `msg` - The text of the message to add
    /// * `date` - The date/id of the message
    ///
    /// # Returns
    ///
    /// Returns the unique identifier of the added message (this is again, the date of that message).
    async fn add_msg(&self, receiver: &str, sent_self: bool, msg: &str, date: u128) -> Result<u128>;
}

#[async_trait]
impl ChatStorageHelper for StorageManager {
    async fn add_msg(&self, receiver: &str, sent_self: bool, msg: &str, date: u128) -> Result<u128> {
        let status = if sent_self { WsMessageStatus::Sending } else { WsMessageStatus::Success };

        // Modifies the storage data and returns the date of the message
        let date = self.modify_storage_data(|e| {
            let c = e
                .chats
                .get_mut(receiver)
                .ok_or(anyhow!("Chat to add message to could not be found"))?;

            c.messages.push(ChatMessage {
                self_sent: sent_self,
                msg: msg.to_string(),
                date,
                status: status.clone()
            });

            Ok(date)
        })
        .await?;

        // Notifies the frontend about the newly created message
        APP_HANDLE.read().await.as_ref().map(|e| e.emit_payload(WsMessageStatusPayload {
            hostname: receiver.to_string(),
            date,
            status
        }));
        Ok(date)
    }
}

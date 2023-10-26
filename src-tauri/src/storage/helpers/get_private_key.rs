use anyhow::Result;

use crate::storage::{STORAGE, StorageChat, StorageManager};
use crate::encryption::PrivateKey;
use async_trait::async_trait;

#[async_trait]
pub trait GetPrivateKey {
    async fn get_or_create_private_key(receiver: &str) -> Result<PrivateKey>;
}

#[async_trait]
impl GetPrivateKey for StorageManager {
    async fn get_or_create_private_key(receiver: &str) -> Result<PrivateKey> {
        let storage = STORAGE.read().await;
        let mut priv_key = storage
            .get_data(|e| {
                let k = e.chats.get(receiver).and_then(|e| Some(e.priv_key.clone()));

                Ok(k)
            })
            .await?;

        if priv_key.is_none() {
            drop(storage);
            let mut storage = STORAGE.write().await;
            priv_key = storage
                .modify_storage_data(|e| {
                    if !e.chats.contains_key(receiver) {
                        e.chats
                            .insert(receiver.to_string(), StorageChat::new(receiver));
                    }

                    let priv_key = e.chats.get(receiver).and_then(|e| Some(e.priv_key.clone()));

                    Ok(priv_key)
                })
                .await?;
        }

        Ok(priv_key.expect("Should always be true"))
    }
}
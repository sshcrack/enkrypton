use anyhow::Result;
use encryption::PrivateKey;
use log::debug;

use async_trait::async_trait;

use crate::{StorageManager, STORAGE, StorageChat};

#[async_trait]
pub trait GetPrivateKey {
    async fn get_or_create_private_key(receiver: &str) -> Result<PrivateKey>;
}

#[async_trait]
impl GetPrivateKey for StorageManager {
    async fn get_or_create_private_key(receiver: &str) -> Result<PrivateKey> {
        debug!("Get or create");
        let mut priv_key = STORAGE.read().await
            .get_data(|e| {
                let k = e.chats.get(receiver).and_then(|e| Some(e.priv_key.clone()));

                Ok(k)
            })
            .await?;

            debug!("Done 1");
        if priv_key.is_none() {
            debug!("None, write");
            priv_key = STORAGE.read().await
                .modify_storage_data(|e| {
                    if !e.chats.contains_key(receiver) {
                        e.chats
                            .insert(receiver.to_string(), StorageChat::new(receiver));
                    }

                    let priv_key = e.chats.get(receiver).and_then(|e| Some(e.priv_key.clone()));

                    Ok(priv_key)
                })
                .await?;
            debug!("Done");
        }

        Ok(priv_key.expect("Should always be true"))
    }
}
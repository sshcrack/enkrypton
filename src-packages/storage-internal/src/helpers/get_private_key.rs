use anyhow::Result;
use encryption::PrivateKey;
use log::info;

use async_trait::async_trait;
use payloads::data::StorageChat;

use crate::{StorageManager, STORAGE};

/// Trait to either get a private key or if it does not exist, generate a new keypair and store it
#[async_trait]
pub trait GetPrivateKey {
    async fn get_or_create_private_key(receiver: &str) -> Result<PrivateKey>;
}

#[async_trait]
impl GetPrivateKey for StorageManager {
    async fn get_or_create_private_key(receiver: &str) -> Result<PrivateKey> {
        // Gets the private key from the storage
        let mut priv_key = STORAGE.read().await
            .get_data(|e| {
                let k = e.chats.get(receiver).and_then(|e| Some(e.priv_key.clone()));

                Ok(k)
            })
            .await?;
        if priv_key.is_none() {
            // If it does not exist, generate a new chat and store it
            priv_key = STORAGE.read().await
                .modify_storage_data(|e| {
                    if !e.chats.contains_key(receiver) {
                        info!("No private key for receiver '{}' yet. Adding new receiver...", receiver);
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
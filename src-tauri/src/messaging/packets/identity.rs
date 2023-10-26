use anyhow::{Result, anyhow};
use log::{warn, info};
use openssl::{pkey::PKey, sign::Verifier};
use serde::{Serialize, Deserialize};

use crate::{storage::{STORAGE, StorageChat}, tor::consts::DIGEST, encryption::PublicKey};


#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub hostname: String,
    pub signature: Vec<u8>,
    pub pub_key: PublicKey,
}

impl Identity {
    /// Fails if not valid
    /// Just passes if valid
    pub async fn verify(&self) -> Result<()> {
        let Identity {  hostname: remote_host, pub_key, signature} = self;

        let storage = STORAGE.read().await;
        let local_pub_key = storage.get_data(|e| {
            let key = e.chats.get(remote_host)
                .and_then(|e| e.pub_key.clone());

            return Ok(key)
        }).await?;

        if let Some(local_pub_key) = local_pub_key {
            info!("Verifying for hostname: {:?}", remote_host);
            let keypair = PKey::from_rsa(local_pub_key.0)?;
            let mut verifier = Verifier::new(*DIGEST, &keypair)?;

            verifier.update(remote_host.as_bytes())?;
            let is_valid = verifier.verify(&signature)?;

            if !is_valid {
                warn!("[INVALID_SIGNATURE] Wrong signature was given! This may be an attack!");
                return Err(anyhow!("Wrong signature was given! This may be an attack!"));
            }

            Ok(())
        } else {
            info!("No chat with hostname '{}' yet. Adding new receiver...", remote_host);
            drop(storage);
            let mut storage = STORAGE.write().await;
            storage.modify_storage_data(|e| {
                let res = e.chats.entry(remote_host.clone())
                    .or_insert_with(|| StorageChat::new(&remote_host));

                res.pub_key = Some(pub_key.clone());

                Ok(())
            }).await?;

            Ok(())
        }
    }
}
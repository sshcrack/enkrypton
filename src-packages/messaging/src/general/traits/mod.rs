mod client_helper;
mod server_helper;

use anyhow::{Result, anyhow};
pub use client_helper::*;
use log::{debug, info, warn};
use openssl::{sign::Verifier, pkey::PKey};
use payloads::packets::Identity;
pub use server_helper::*;
use storage_internal::{STORAGE, StorageChat};
use encryption::consts::DIGEST;

#[async_trait::async_trait]
pub trait IdentityProvider<T> {
    async fn identity(receiver: &str) -> Result<T>;
}



#[async_trait::async_trait]
pub trait IdentityVerify {
    async fn verify(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl IdentityVerify for Identity {
    async fn verify(&self) -> Result<()> {
        let Identity {  hostname: remote_host, pub_key, signature} = self;

        debug!("Reading to verify...");
        let local_pub_key = STORAGE.read().await.get_data(|e| {
            let key = e.chats.get(remote_host)
                .and_then(|e| e.rec_pub_key.clone());

            return Ok(key)
        }).await?;

        debug!("Done");

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
            STORAGE.read().await.modify_storage_data(|e| {
                let res = e.chats.entry(remote_host.clone())
                    .or_insert_with(|| StorageChat::new(&remote_host));

                res.rec_pub_key = Some(pub_key.clone());

                Ok(())
            }).await?;
            debug!("Done.");

            Ok(())
        }
    }
}

mod client_helper;
mod server_helper;

use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use openssl::{sign::Verifier, pkey::PKey};
use payloads::{packets::Identity, data::StorageChat};
use storage_internal::STORAGE;
use encryption::consts::DIGEST;

#[async_trait::async_trait]
pub trait IdentityProvider<T> {
    /// Returns the identity with the given receiver, look at the implementations for more detail
    async fn identity(receiver: &str) -> Result<T>;
}



#[async_trait::async_trait]
pub trait IdentityVerify {
    /// Verifies the identity and stores public keys in storage if needed
    async fn verify(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl IdentityVerify for Identity {
    async fn verify(&self) -> Result<()> {
        let Identity {  hostname: remote_host, pub_key, signature} = self;

        debug!("Reading to verify...");
        // Check if there is a public key for the given receiver
        let local_pub_key = STORAGE.read().await.get_data(|e| {
            let key = e.chats.get(remote_host)
                .and_then(|e| e.rec_pub_key.clone());

            return Ok(key)
        }).await?;

        debug!("Done");

        // If there is a public key, verify the signature
        if let Some(local_pub_key) = local_pub_key {
            info!("Verifying for hostname: {:?}", remote_host);
            let keypair = PKey::from_rsa(local_pub_key.0)?;
            let mut verifier = Verifier::new(*DIGEST, &keypair)?;

            // Verify the signature with the public key
            verifier.update(remote_host.as_bytes())?;
            let is_valid = verifier.verify(&signature)?;

            if !is_valid {
                warn!("[INVALID_SIGNATURE] Wrong signature was given! This may be an attack!");
                return Err(anyhow!("Wrong signature was given! This may be an attack!"));
            }

            Ok(())
        } else {
            // Adding public key to storage because it does  not exist
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

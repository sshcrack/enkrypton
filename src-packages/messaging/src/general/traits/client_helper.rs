use anyhow::{anyhow, Result};
use openssl::{pkey::PKey, sign::Signer};
use payloads::packets::{C2SPacket, Identity};
use storage_internal::{StorageManager, helpers::GetPrivateKey};
use tor_proxy::service::get_service_hostname;
use encryption::consts::DIGEST;

use super::IdentityProvider;


#[async_trait::async_trait]
impl IdentityProvider<Self> for C2SPacket {
    async fn identity(receiver: &str) -> Result<Self> {
        // Get the own hostname
        let own_hostname = get_service_hostname(true)
            .await?
            .ok_or(anyhow!("Could not get own hostname"))?;

        // Get the private key for the receiver (used to decrypt messages)
        let priv_key = StorageManager::get_or_create_private_key(receiver).await?;
        let pub_key = priv_key.clone().try_into()?;

        // Creating a signature for the receiver with the hostname
        let keypair = PKey::from_rsa(priv_key.0)?;
        let mut signer = Signer::new(*DIGEST, &keypair)?;

        signer.update(own_hostname.as_bytes())?;
        let signature = signer.sign_to_vec()?;

        // Return the identity packet
        Ok(C2SPacket::SetIdentity(Identity {
            hostname: own_hostname,
            signature,
            pub_key
        }))
    }
}
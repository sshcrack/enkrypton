use super::IdentityProvider;
use anyhow::{Result, anyhow};
use encryption::{consts::DIGEST, PrivateKey};
use openssl::{pkey::PKey, sign::Signer};
use payloads::packets::{Identity, S2CPacket};
use storage_internal::{StorageManager, helpers::GetPrivateKey};
use tor_proxy::service::get_service_hostname;


#[async_trait::async_trait]
impl IdentityProvider<Self> for S2CPacket {
    async fn identity(receiver: &str) -> Result<Self> {
        let own_hostname = get_service_hostname(false)
            .await?
            .ok_or(anyhow!("Could not get own hostname"))?;

        let priv_key: PrivateKey = StorageManager::get_or_create_private_key(receiver).await?;
        let pub_key = priv_key.clone().try_into()?;

        let keypair = PKey::from_rsa(priv_key.0)?;
        let mut signer = Signer::new(*DIGEST, &keypair)?;

        signer.update(own_hostname.as_bytes())?;
        let signature = signer.sign_to_vec()?;

        Ok(S2CPacket::VerifyIdentity(Identity {
            hostname: own_hostname,
            signature,
            pub_key,
        }))
    }
}
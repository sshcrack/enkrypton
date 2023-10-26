use anyhow::{anyhow, Result};
use openssl::{pkey::PKey, sign::Signer};
use serde::{Deserialize, Serialize};

use crate::{tor::{service::get_service_hostname, consts::DIGEST}, storage::{StorageManager, helpers::GetPrivateKey}};

use super::Identity;

#[derive(Serialize, Deserialize)]
pub enum S2CPacket {
    DisconnectMultipleConnections,
    VerifyIdentity(Identity),
    IdentityVerified,
    Message(Vec<u8>),
}

impl S2CPacket {
    pub async fn identity(receiver: &str) -> Result<Self> {
        let own_hostname = get_service_hostname()
            .await?
            .ok_or(anyhow!("Could not get own hostname"))?;

        let priv_key = StorageManager::get_or_create_private_key(receiver).await?;
        let pub_key = priv_key.clone().try_into()?;

        let keypair = PKey::from_rsa(priv_key.0)?;
        let mut signer = Signer::new(*DIGEST, &keypair)?;

        signer.update(own_hostname.as_bytes())?;
        let signature = signer.sign_to_vec()?;

        Ok(S2CPacket::VerifyIdentity(Identity {
            hostname: receiver.to_string(),
            signature,
            pub_key,
        }))
    }
}

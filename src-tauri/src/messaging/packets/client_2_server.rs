use anyhow::{anyhow, Result};
use openssl::{pkey::PKey, sign::Signer};
use serde::{Deserialize, Serialize};

use crate::{
    storage::{
        helpers::GetPrivateKey, StorageManager,
    },
    tor::{service::get_service_hostname, consts::DIGEST},
};

use super::Identity;


#[derive(Debug, Serialize, Deserialize)]
pub enum C2SPacket {
    SetIdentity(Identity),
    IdentityVerified,
    Message(Vec<u8>),
}

impl C2SPacket {
    pub async fn identity(receiver: &str) -> Result<Self> {
        let own_hostname = get_service_hostname(true)
            .await?
            .ok_or(anyhow!("Could not get own hostname"))?;

        let priv_key = StorageManager::get_or_create_private_key(receiver).await?;
        let pub_key = priv_key.clone().try_into()?;

        let keypair = PKey::from_rsa(priv_key.0)?;
        let mut signer = Signer::new(*DIGEST, &keypair)?;

        signer.update(own_hostname.as_bytes())?;
        let signature = signer.sign_to_vec()?;

        Ok(C2SPacket::SetIdentity(Identity { hostname: own_hostname.to_string(), signature, pub_key }))
    }
}

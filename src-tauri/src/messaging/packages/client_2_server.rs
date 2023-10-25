use anyhow::{anyhow, Result};
use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};
use serde::{Deserialize, Serialize};

use crate::{storage::STORAGE, tor::service::get_service_hostname};


#[derive(Serialize, Deserialize)]
pub enum C2SPacket {
    VerifyIdentity(Vec<u8>),
}

impl C2SPacket {
    pub async fn verify() -> Result<Self> {
        let storage = STORAGE.read().await;
        let own_hostname = get_service_hostname()
            .await?
            .ok_or(anyhow!("Could not get own hostname"))?;

        let signature = storage
            .get_data(|e| {
                let keypair = PKey::from_rsa(e.priv_key.0.clone())?;
                let mut signer = Signer::new(MessageDigest::sha256(), &keypair)?;

                signer.update(own_hostname.as_bytes())?;
                Ok(signer.sign_to_vec()?)
            })
            .await?;

        Ok(C2SPacket::VerifyIdentity(signature))
    }
}
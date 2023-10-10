use std::fmt::Debug;

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::PasswordHashString,
    Argon2, PasswordVerifier,
};
use byteorder::{WriteBytesExt, LE};

use crate::{
    consts::{FILE_ID_BYTES, KEY_LENGTH},
    encryption::aes::{aes_decrypt, aes_encrypt},
};

#[derive(Debug)]
pub struct RawStorageData<T> {
    pub(super) pass_hash: PasswordHashString,
    pub(super) iv: Box<[u8]>,
    pub(super) encrypted_data: Box<[u8]>,

    pub(super) crypto_key: Option<Box<[u8]>>,
    pub(super) data: Option<T>,
}

impl<T> RawStorageData<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug,
{
    fn decrypt_crypto_key(&mut self, password: &[u8]) -> Result<()> {
        let argon2 = Argon2::default();

        let hash = self.pass_hash.password_hash();
        let valid = argon2.verify_password(password, &hash);
        if valid.is_err() {
            return Err(anyhow!(valid.unwrap_err().to_string()));
        }

        let mut crypto_key = vec![0u8; *KEY_LENGTH];

        //TODO really this?
        let salt_raw = hash.salt.unwrap().as_str().as_bytes();

        argon2
            .hash_password_into(password, &salt_raw, &mut crypto_key) //
            .or_else(|e| Err(anyhow!(e.to_string())))?;

        self.crypto_key = Some(crypto_key.into_boxed_slice());
        Ok(())
    }

    pub(super) fn decrypt(&mut self, password: &[u8]) -> Result<()> {
        self.decrypt_crypto_key(password)?;
        let crypto_key = self
            .crypto_key
            .as_ref()
            .expect("Why did this come through?");

        let decrypted = aes_decrypt(&self.encrypted_data, crypto_key, &self.iv)?;
        let parsed: T = serde_json::from_slice(&decrypted)?;

        println!("Decrypted data {:#?}", parsed);
        self.data = Some(parsed);
        Ok(())
    }

    /***
     * Overwrites raw_data by encrypting data
     */
    pub fn update_raw(&mut self) -> Result<()> {
        if self.data.is_none() {
            return Err(anyhow!("No data to encrypt"));
        }

        if self.crypto_key.is_none() {
            return Err(anyhow!("No crypto key set."));
        }

        let key = self.crypto_key.as_ref().unwrap();

        let d = self.data.as_ref().unwrap();
        let serialized = serde_json::to_string(d)?;

        let encrypted = aes_encrypt(serialized.as_bytes(), key, &self.iv)?;

        self.encrypted_data = encrypted.into_boxed_slice();

        Ok(())
    }

    pub fn to_raw(&mut self) -> Result<Vec<u8>> {
        self.update_raw()?;

        let hash = &self.pass_hash;

        let mut raw = Vec::new();
        raw.extend_from_slice(&FILE_ID_BYTES);
        raw.write_u64::<LE>(hash.len() as u64)?;
        raw.extend_from_slice(hash.as_bytes());
        raw.extend_from_slice(&self.iv);
        raw.extend_from_slice(&self.encrypted_data);

        Ok(raw)
    }
}

use std::fmt::Debug;

use anyhow::{Result, anyhow};
use argon2::{Argon2, password_hash::{SaltString, rand_core::OsRng}, PasswordHasher};
use openssl::rand::rand_bytes;

use crate::{RawStorageData, consts::{IV_LENGTH, KEY_LENGTH}};

pub trait Generate<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug,
{
    fn generate(pass: &[u8], data: T) -> Result<RawStorageData<T>>;
}

impl<T> Generate<T> for RawStorageData<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug,
{
    fn generate(pass: &[u8], data: T) -> Result<RawStorageData<T>> {
        let mut iv = vec![0u8; *IV_LENGTH];
        rand_bytes(&mut iv)?;

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let pass_hash = argon2.hash_password(pass, &salt)?;

        let mut crypto_key = vec![0u8; *KEY_LENGTH];

        //TODO really this?
        let salt_raw = salt.as_str().as_bytes();

        argon2
            .hash_password_into(&pass, &salt_raw, &mut crypto_key) //
            .or_else(|e| Err(anyhow!(e.to_string())))?;

        let crypto_key = crypto_key.into_boxed_slice();
        let iv = iv.into_boxed_slice();
        let pass_hash_str = pass_hash.serialize();

        let mut constructed = Self {
            crypto_key: Some(crypto_key),
            data: Some(data),
            encrypted_data: Vec::new().into_boxed_slice(),
            iv,
            pass_hash: pass_hash_str,
        };

        constructed.update_raw()?;

        Ok(constructed)
    }
}

use std::fmt::Debug;

use anyhow::{bail, Result};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use openssl::rand::rand_bytes;
use zeroize::Zeroize;

use crate::{
    consts::{IV_LENGTH, KEY_LENGTH},
    Errors, SecureStorage,
};

pub trait Generate<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    fn generate(pass: &[u8], data: T) -> Result<SecureStorage<T>>;
}

impl<T> Generate<T> for SecureStorage<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    fn generate(pass: &[u8], data: T) -> Result<SecureStorage<T>> {
        let mut iv = vec![0u8; *IV_LENGTH];
        rand_bytes(&mut iv).or_else(|e| bail!(Errors::RandomIV(e)))?;

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let pass_hash = argon2
            .hash_password(pass, &salt)
            .or_else(|e| bail!(Errors::GenerateHash(e)))?;

        let mut crypto_key = vec![0u8; *KEY_LENGTH];

        //TODO really this?
        let salt_raw = salt.as_str().as_bytes();

        argon2
            .hash_password_into(&pass, &salt_raw, &mut crypto_key) //
            .or_else(|e| bail!(Errors::CryptoKeyError(e)))?;

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

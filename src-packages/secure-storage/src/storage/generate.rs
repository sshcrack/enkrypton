use std::fmt::Debug;

use anyhow::{bail, Result};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString, Salt},
    Argon2, PasswordHasher,
};
use openssl::rand::rand_bytes;
use zeroize::Zeroize;

use crate::{
    consts::{IV_LENGTH, KEY_LENGTH},
    Errors, SecureStorage,
};

/// A trait to extend the SecureStorage with a function to generate a new one
pub trait Generate<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    /// 
    /// Generates a new SecureStorage with the given password and data
    ///
    /// # Arguments
    ///
    /// * `pass` - The password that should be used to encrypt the data
    /// * `data` - Default data to use
    ///
    /// # Returns
    ///
    /// The generated `SecureStorage` used to store data
    fn generate(pass: &[u8], data: T) -> Result<SecureStorage<T>>;
}

impl<T> Generate<T> for SecureStorage<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    fn generate(pass: &[u8], data: T) -> Result<SecureStorage<T>> {
        let mut iv = vec![0u8; *IV_LENGTH];
        // Generate a new random iv
        rand_bytes(&mut iv).or_else(|e| bail!(Errors::RandomIV(e)))?;

        // Hashing the password using the argon2 algorithm
        let argon2 = Argon2::default();

        // But firstly we need a string
        let salt_str = SaltString::generate(&mut OsRng);
        let salt = Salt::from(&salt_str);

        // And then we'll hash it
        let pass_hash = argon2
            .hash_password(pass, salt)
            .map_err(|e| Errors::GenerateHash(e))?;

        // Now we need to hash the password to get the crypto key
        let mut crypto_key = vec![0u8; *KEY_LENGTH];

        //NOTE seems scuffed but actually works so we take it alright
        let salt_raw = salt.as_str().as_bytes();

        // And we obtain the crypto key
        argon2
            .hash_password_into(&pass, &salt_raw, &mut crypto_key) //
            .map_err(|e| Errors::CryptoKeyError(e))?;

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

        // And write stuff to the disk
        constructed.update_raw()?;

        Ok(constructed)
    }
}

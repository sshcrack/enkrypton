use std::fmt::Debug;

use anyhow::{anyhow, bail, Result};
use argon2::{password_hash::PasswordHashString, Argon2, PasswordVerifier};
use byteorder::{WriteBytesExt, LE};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    consts::{FILE_ID_BYTES, KEY_LENGTH},
    encryption::aes::{aes_decrypt, aes_encrypt},
    Errors,
};

/// Holds all data that is stored on the disk as well as crypto_keys and pass_hash
#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct SecureStorage<T: zeroize::Zeroize> {
    /// The password hash that is used to verify the password
    #[zeroize(skip)]
    pub(super) pass_hash: PasswordHashString,
    /// The IV used to encrypt data with AES
    pub(super) iv: Box<[u8]>,
    /// The encrypted data
    pub(super) encrypted_data: Box<[u8]>,

    /// The crypto key used to encrypt/decrypt the data
    pub(super) crypto_key: Option<Box<[u8]>>,
    /// And the data itself which is decrypted
    pub data: Option<T>,
}

#[allow(clippy::wrong_self_convention)]
impl<T> SecureStorage<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    /// Tries to decrypt the crypto key with the given password
    ///
    /// # Arguments
    ///
    /// * `password` - The password in binary (encoded in utf8
    fn try_decrypt_crypto_key(&mut self, password: &[u8]) -> Result<()> {
        let argon2 = Argon2::default();

        // Get the hash
        let hash = self.pass_hash.password_hash();
        // Verify the actual password
        self.verify_password(password)?;

        let mut crypto_key = vec![0u8; *KEY_LENGTH];

        //NOTE seems scuffed but actually works so we take it alright
        let salt_raw = hash.salt.unwrap().as_str().as_bytes();

        // And we obtain the crypto key
        argon2
            .hash_password_into(password, &salt_raw, &mut crypto_key) //
            .or_else(|e| bail!(Errors::CryptoKeyError(e)))?;

        self.crypto_key = Some(crypto_key.into_boxed_slice());
        Ok(())
    }

    /// Tries to decrypt the data with the given password
    ///
    /// # Arguments
    ///
    /// * `password` - The password in binary (again, encoded in utf8)
    pub fn try_decrypt(&mut self, password: &[u8]) -> Result<()> {
        // First, we need to decrypt the crypto key
        self.try_decrypt_crypto_key(password)?;
        let crypto_key = self
            .crypto_key
            .as_ref()
            .expect("Why did this come through?");

        // Then we can decrypt the data using the crypto key
        let decrypted = aes_decrypt(&self.encrypted_data, crypto_key, &self.iv)?;

        // And parse the actual data
        let parsed: T = serde_json::from_slice(&decrypted) //.
            .or_else(|e| bail!(Errors::JsonParse(e)))?;

        self.data = Some(parsed);
        Ok(())
    }

    /// Encrypts `data` and stores it as `encrypted_data`. Overwrites raw encrypted data
    pub(super) fn update_raw(&mut self) -> Result<()> {
        if self.data.is_none() {
            return Err(anyhow!("No data to encrypt"));
        }

        if self.crypto_key.is_none() {
            return Err(anyhow!("No crypto key set."));
        }

        let key = self.crypto_key.as_ref().unwrap();

        let d = self.data.as_ref().unwrap();
        // Serialize the data
        let serialized = serde_json::to_string(d) //.
            .or_else(|e| bail!(Errors::JsonSerialize(e)))?;

        // And encrypts the data with the crypto key
        let encrypted = aes_encrypt(serialized.as_bytes(), key, &self.iv)?;

        self.encrypted_data = encrypted.into_boxed_slice();
        Ok(())
    }

    /// # Returns
    /// 
    /// the data that should be written to disk, including hashes, iv and encrypted_data for example
    pub fn to_raw(&mut self) -> Result<Vec<u8>> {
        self.update_raw()?;

        let hash = &self.pass_hash;

        // Always start with the given file id
        let mut raw = Vec::new();
        raw.extend_from_slice(&FILE_ID_BYTES);
        raw.write_u64::<LE>(hash.len() as u64)?;
        raw.extend_from_slice(hash.as_bytes());
        raw.extend_from_slice(&self.iv);
        raw.extend_from_slice(&self.encrypted_data);

        Ok(raw)
    }

    /// Check if the password is valid, fails if not
    ///
    /// # Arguments
    ///
    /// * `pass` - The password in binary (encoded in utf8)
    pub fn verify_password(&self, pass: &[u8]) -> Result<()> {
        let argon = Argon2::default();
        let h = self.pass_hash.password_hash();

        argon
            .verify_password(pass, &h)
            .or_else(|e| bail!(Errors::PasswordError(e)))
    }
}

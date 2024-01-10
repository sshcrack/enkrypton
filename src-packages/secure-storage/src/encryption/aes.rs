use anyhow::{bail, Result};
use openssl::symm::{decrypt, encrypt};

use crate::{consts::CIPHER, Errors};

/// Used to encrypt the given data with a key and an iv using the CIPHER and AES
///
/// # Arguments
///
/// * `data` - the data to encrypt
/// * `key` - the public key that should be used to encrypt this data
/// * `iv` - The iv of the encryption
///
/// # Returns
///
/// The encrypted data
pub fn aes_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    encrypt(*CIPHER, key, Some(&iv), data) //.
        .or_else(|e| bail!(Errors::AESEncrypt(e)))
}

/// Used to decrypt the given data with a key and an iv using the CIPHER and AES
///
/// # Arguments
///
/// * `data` - the data to decrypt
/// * `key` - the private key that should be used to decrypt this data
/// * `iv` - The iv of the decryption
///
/// # Returns
///
/// The decrypted data
pub fn aes_decrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    decrypt(*CIPHER, key, Some(iv), data) //.
        .or_else(|e| bail!(Errors::AESDecrypt(e)))
}

use anyhow::{bail, Result};
use openssl::symm::{decrypt, encrypt};

use crate::{consts::CIPHER, Errors};

/// Used to encrypt the given data with a key and an iv using the CIPHER and AES
pub fn aes_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    encrypt(*CIPHER, key, Some(&iv), data) //.
        .or_else(|e| bail!(Errors::AESEncrypt(e)))
}

/// Used to decrypt with AES and the given key and iv
pub fn aes_decrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    decrypt(*CIPHER, key, Some(iv), data) //.
        .or_else(|e| bail!(Errors::AESDecrypt(e)))
}

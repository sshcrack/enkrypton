use anyhow::Result;
use openssl::symm::{decrypt, encrypt};

use crate::consts::CIPHER;

pub fn aes_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let encrypted = encrypt(*CIPHER, key, Some(&iv), data)?;

    Ok(encrypted)
}

pub fn aes_decrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let decrypted = decrypt(*CIPHER, key, Some(iv), data)?;

    Ok(decrypted)
}

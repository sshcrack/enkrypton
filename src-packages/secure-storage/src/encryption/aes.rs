use anyhow::{bail, Result};
use openssl::symm::{decrypt, encrypt};

use crate::{consts::CIPHER, Errors};

pub fn aes_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    encrypt(*CIPHER, key, Some(&iv), data) //.
        .or_else(|e| bail!(Errors::AESEncrypt(e)))
}

pub fn aes_decrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    decrypt(*CIPHER, key, Some(iv), data) //.
        .or_else(|e| bail!(Errors::AESDecrypt(e)))
}

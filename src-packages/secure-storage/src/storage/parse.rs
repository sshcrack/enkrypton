use std::fmt::Debug;

use anyhow::{anyhow, Result};
use argon2::password_hash::{Encoding, PasswordHashString};
use byteorder::{ReadBytesExt, LE};
use zeroize::Zeroize;

use crate::{consts::{IV_LENGTH, FILE_ID_BYTES}, SecureStorage};

const U64_BYTES: usize = u64::BITS as usize / 8usize;

pub trait Parsable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    fn parse(raw: &[u8], pass: &[u8]) -> Result<SecureStorage<T>>;
}

impl<T> Parsable<T> for SecureStorage<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + Debug + Zeroize,
{
    fn parse(raw: &[u8], pass: &[u8]) -> Result<SecureStorage<T>> {
        let mut buffer = Vec::from(raw);
        if raw.len() < FILE_ID_BYTES.len() {
            return Err(anyhow!("Could not parse file, too short"));
        }

        for i in 0..FILE_ID_BYTES.len() {
            let curr = buffer.remove(0);
            if curr != FILE_ID_BYTES[i] {
                panic!("Invalid file format");
            }
        }

        if buffer.len() < U64_BYTES {
            return Err(anyhow!("Could not read hash size, too short"));
        }

        let hash_size: Vec<u8> = buffer.drain(0..U64_BYTES).collect();
        let hash_size = hash_size.as_slice().read_u64::<LE>()? as usize;

        if buffer.len() < hash_size {
            return Err(anyhow!("Could read hash, too short"));
        }

        let hash: Vec<u8> = buffer.drain(0..hash_size).collect();
        let hash = String::from_utf8(hash)?;
        let hash = PasswordHashString::parse(&hash, Encoding::B64) //
            .or_else(|e| Err(anyhow!(e.to_string())))?;

        if buffer.len() < *IV_LENGTH {
            return Err(anyhow!("Could not parse iv, file too short"));
        }

        let iv = buffer.drain(0..*IV_LENGTH).collect();
        let mut constructed = SecureStorage {
            pass_hash: hash,
            iv,
            encrypted_data: buffer.into_boxed_slice(),
            crypto_key: None,
            data: None,
        };

        constructed.decrypt(pass)?;

        Ok(constructed)
    }
}

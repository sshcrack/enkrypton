pub mod consts;
#[cfg(test)]
mod tests;

use anyhow::Result;
use consts::{RSA_KEY_SIZE, RSA_PADDING};
use openssl::{
    encrypt::{Decrypter, Encrypter},
    error::ErrorStack,
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

/// Just a wrapper to the openssl RSA Key. Used for serialization and deserialization.
#[derive(Clone, Debug)]
pub struct PrivateKey(pub Rsa<Private>);

/// Serialize the private key to a PEM string
impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self
            .0
            .private_key_to_pem()
            .map_err(|e| ser::Error::custom(e.to_string()))?;
        serializer.serialize_bytes(&bytes)
    }
}

/// Parses the public key from a PEM string (String is given in Vec<u8> utf8 bytes)
impl<'a> Deserialize<'a> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        // We are deserializing the key from bytes and adding it to the deserializer
        <Vec<u8>>::deserialize(deserializer).and_then(|s| {
            Rsa::private_key_from_pem(&s)
                .and_then(|e| Ok(PrivateKey(e)))
                .map_err(|e| de::Error::custom(e.to_string()))
        })
    }
}

/// A wrapper for the openssl key struct. Same as the private key wrapper.
#[derive(Clone, Debug)]
pub struct PublicKey(pub Rsa<Public>);

/// Converts the private key to a public key by converting it to a pem and then back to a public key
impl TryInto<PublicKey> for PrivateKey {
    type Error = ErrorStack;

    fn try_into(self) -> Result<PublicKey, Self::Error> {
        let pem = self.0.public_key_to_pem()?;
        // Again, we are just converting the pem to a public key
        Rsa::public_key_from_pem(pem.as_slice()).map(|e| PublicKey(e))
    }
}

/// Serializes the public key to a PEM string
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // We are serializing the key to bytes and adding it to the serializer
        let bytes = self
            .0
            .public_key_to_pem()
            .map_err(|e| ser::Error::custom(e.to_string()))?;
        serializer.serialize_bytes(&bytes)
    }
}

/// Deserializes from a PEM string to a public key
impl<'a> Deserialize<'a> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        <Vec<u8>>::deserialize(deserializer).and_then(|s| {
            Rsa::public_key_from_pem(&s)
                .and_then(|e| Ok(PublicKey(e)))
                .map_err(|e| de::Error::custom(e.to_string()))
        })
    }
}

impl PrivateKey {
    /// Generates a new RSA key pair with the key size of RSA_KEY_SIZE.
    ///
    /// # Returns
    ///
    /// The generated RSA key pair.
    ///
    /// # Returns
    ///
    /// The generated RSA key pair.
    pub fn generate_pair() -> Result<Self> {
        let res = Rsa::generate(*RSA_KEY_SIZE)?;

        Ok(Self(res))
    }

    /// Decrypts the given data with the given private key.
    ///
    /// # Arguments
    ///
    /// * `encrypted` - The encrypted data to decrypt.
    /// * `encrypted` - The private key
    ///
    /// # Returns
    ///
    ///
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        let key = PKey::from_rsa(self.0.clone())?;

        let mut decrypter = Decrypter::new(&key)?;
        decrypter.set_rsa_padding(*RSA_PADDING)?;

        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(encrypted)?;
        let mut decrypted = vec![0; buffer_len];

        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(encrypted, &mut decrypted)?;
        decrypted.truncate(decrypted_len);

        Ok(decrypted)
    }
}

impl PublicKey {
    /// Encrypts the given data with the given public key.
    ///
    /// # Arguments
    ///
    /// * `data` - A vector of bytes to encrypt.
    /// * `key` - The public key to encrypt the data with.
    ///
    /// # Returns
    /// The encrypted data.
    ///
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.0.clone();

        // Generate a keypair
        let key = PKey::from_rsa(key)?;

        // Encrypt the data with RSA PKCS1
        let mut encrypter = Encrypter::new(&key)?;
        encrypter.set_rsa_padding(*RSA_PADDING)?;

        // Create an output buffer
        let buffer_len = encrypter.encrypt_len(data)?;
        let mut encrypted = vec![0; buffer_len];

        // Encrypt and truncate the buffer
        let encrypted_len = encrypter.encrypt(data, &mut encrypted)?;
        encrypted.truncate(encrypted_len);

        Ok(encrypted)
    }
}
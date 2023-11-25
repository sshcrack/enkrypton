pub mod consts;

use anyhow::Result;
use openssl::{
    error::ErrorStack,
    pkey::{Private, Public, PKey},
    rsa::Rsa, encrypt::{Encrypter, Decrypter},
};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use consts::{RSA_PADDING, RSA_KEY_SIZE};

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

/// Deserialize the private key from a PEM string
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

/// Well, a private key can be converted into a public key so we do that here.
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

/// Generates a brand new key pair with 4096 bits
pub fn generate_pair() -> PrivateKey {
    // Generating the actual keypair
    let res = Rsa::generate(*RSA_KEY_SIZE).unwrap();

    PrivateKey(res)
}

///Encrypting the data with the given key
pub fn rsa_encrypt(data: Vec<u8>, key: &PublicKey) -> Result<Vec<u8>> {
    let key = key.0.clone();

    // Generate a keypair
    let key = PKey::from_rsa(key)?;

    // Encrypt the data with RSA PKCS1
    let mut encrypter = Encrypter::new(&key)?;
    encrypter.set_rsa_padding(*RSA_PADDING)?;

    // Create an output buffer
    let buffer_len = encrypter.encrypt_len(&data)?;
    let mut encrypted = vec![0; buffer_len];

    // Encrypt and truncate the buffer
    let encrypted_len = encrypter.encrypt(&data, &mut encrypted)?;
    encrypted.truncate(encrypted_len);

    Ok(encrypted)
}

/// Decrypting the data with the given openssl key
pub fn rsa_decrypt(encrypted: Vec<u8>, key: PrivateKey) -> Result<Vec<u8>> {
    let key = PKey::from_rsa(key.0.clone())?;

    let mut decrypter = Decrypter::new(&key)?;
    decrypter.set_rsa_padding(*RSA_PADDING)?;

    // Create an output buffer
    let buffer_len = decrypter.decrypt_len(&encrypted)?;
    let mut decrypted = vec![0; buffer_len];

    // Encrypt and truncate the buffer
    let decrypted_len = decrypter.decrypt(&encrypted, &mut decrypted)?;
    decrypted.truncate(decrypted_len);

    Ok(decrypted)
}
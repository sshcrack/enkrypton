use openssl::{
    pkey::{Private, Public},
    rsa::Rsa,
};
use serde::{
    de::{self, DeserializeOwned},
    ser, Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Clone, Debug)]
pub struct PrivateKey(Rsa<Private>);

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

impl <'a> Deserialize<'a> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        <Vec<u8>>::deserialize(deserializer).and_then(|s| {
            Rsa::private_key_from_pem(&s)
            .and_then(|e| Ok(PrivateKey(e)))
            .map_err(|e| de::Error::custom(e.to_string()))
        })
    }
}


#[derive(Clone, Debug)]
pub struct PublicKey(Rsa<Public>);

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self
            .0
            .public_key_to_pem()
            .map_err(|e| ser::Error::custom(e.to_string()))?;
        serializer.serialize_bytes(&bytes)
    }
}

impl <'a> Deserialize<'a> for PublicKey {
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


pub fn generate_pair() -> PrivateKey {
    let res = Rsa::generate(4096).unwrap();

    PrivateKey(res)
}

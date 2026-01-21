use anyhow::Result;

use crate::{PrivateKey, PublicKey};

#[test]
fn generate() -> Result<()> {
    PrivateKey::generate_pair()?;
    Ok(())
}

#[test]
fn encrypt_decrypt_test() -> Result<()> {
    let priv_key = PrivateKey::generate_pair()?;
    let msg = "This is my super secret message";

    let public: PublicKey = priv_key.clone().try_into()?;
    let encrypted = public.encrypt(msg.as_bytes())?;

    let decrypted = priv_key.decrypt(&encrypted)?;
    let decrypted = String::from_utf8(decrypted)?;

    assert_eq!(msg, decrypted);
    Ok(())
}

#[test]
fn serialize() -> Result<()> {
    let pair = PrivateKey::generate_pair()?;
    let original_pub: PublicKey = pair.clone().try_into()?;

    let serialized = serde_json::to_string(&pair)?;
    let _deserialized: PrivateKey = serde_json::from_str(&serialized)?;

    let serialized = serde_json::to_string(&original_pub)?;
    let _deserialized: PublicKey = serde_json::from_str(&serialized)?;

    //REVIEW - Maybe check if key was properly deserialized

    Ok(())
}

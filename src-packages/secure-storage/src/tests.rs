use anyhow::Result;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{Generate, Parsable, SecureStorage};

#[derive(Deserialize, Serialize, Debug, Clone, Zeroize, PartialEq, Eq, PartialOrd, Ord)]
struct TestStruct {
    hi: u64,
    lol: String
}

impl Default for TestStruct {
    fn default() -> Self {
        Self { hi: 124124, lol: "hi".to_string() }
    }
}

const PASS: &[u8] = b"VerySecurePassword123";
fn generate() -> Result<SecureStorage<TestStruct>> {

    SecureStorage::generate(PASS, TestStruct::default())
}

#[test]
fn generate_t() -> Result<()> {
    generate()?;

    Ok(())
}


#[test]
fn decrypt() -> Result<()> {
    let mut storage = generate()?;
    let raw = storage.to_raw().unwrap();

    let mut new_str = SecureStorage::<TestStruct>::parse(&raw).unwrap();
    new_str.try_decrypt(PASS).unwrap();

    assert_eq!(storage.data.as_ref().unwrap(), new_str.data.as_ref().unwrap());
    Ok(())
}
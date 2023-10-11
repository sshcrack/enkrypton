#[derive(Debug)]
pub enum Errors {
    PasswordError(argon2::password_hash::Error),
    ParsePassword(argon2::password_hash::Error),
    CryptoKeyError(argon2::Error),

    AESDecrypt(openssl::error::ErrorStack),
    AESEncrypt(openssl::error::ErrorStack),

    JsonParse(serde_json::Error),
    JsonSerialize(serde_json::Error),


    RandomIV(openssl::error::ErrorStack),
    GenerateHash(argon2::password_hash::Error)
}

impl std::error::Error for Errors {}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PasswordError(err) => write!(f, "Could not verify password: {}", err),
            Self::CryptoKeyError(err) => write!(f, "Could not hash password for crypto key: {}", err),
            Self::ParsePassword(err) => write!(f, "Could not parse password hash: {}", err),

            Self::AESDecrypt(err) => write!(f, "Could not decrypt aes: {}", err),
            Self::AESEncrypt(err) => write!(f, "Could not encrypt aes: {}", err),
            Self::JsonParse(err) => write!(f, "Could not parse json: {}", err),
            Self::JsonSerialize(err) => write!(f, "Could not serialize json: {}", err),
            Self::RandomIV(err) => write!(f, "Could not generate random iv: {}", err),
            Self::GenerateHash(err) => write!(f, "Could not generate password hash: {}", err)
        }
    }
}

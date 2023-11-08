use lazy_static::lazy_static;
use openssl::{hash::MessageDigest, rsa::Padding};

lazy_static! {
    /// Default hash used for signing / verifying
    pub static ref DIGEST: MessageDigest = MessageDigest::md5();
    /// Default Padding for RSA encryption
    pub static ref RSA_PADDING: Padding = Padding::PKCS1;
}
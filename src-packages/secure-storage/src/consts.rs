use lazy_static::lazy_static;
use openssl::symm::Cipher;


lazy_static! {
    pub static ref FILE_ID: &'static str = "[secure-storage]";
    pub static ref FILE_ID_BYTES: &'static [u8] = FILE_ID.as_bytes();


    pub static ref CIPHER: Cipher = Cipher::aes_256_cbc();
    pub static ref KEY_LENGTH: usize = CIPHER.key_len();
    pub static ref IV_LENGTH: usize = CIPHER.iv_len().unwrap();
}

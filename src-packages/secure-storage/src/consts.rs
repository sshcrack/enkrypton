use lazy_static::lazy_static;
use openssl::symm::Cipher;


lazy_static! {
    /// The file id every storage file starts with
    pub static ref FILE_ID: &'static str = "[secure-storage]";
    // Just the byte length of it
    pub static ref FILE_ID_BYTES: &'static [u8] = FILE_ID.as_bytes();


    // The cipher we are going to use
    pub static ref CIPHER: Cipher = Cipher::aes_256_cbc();
    // Key length of the cipher
    pub static ref KEY_LENGTH: usize = CIPHER.key_len();
    // And the iv length that is needed
    pub static ref IV_LENGTH: usize = CIPHER.iv_len().unwrap();
}

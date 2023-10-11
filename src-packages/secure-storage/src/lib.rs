mod encryption;
mod storage;
mod consts;
mod error;

pub use error::*;
pub use storage::*;


#[cfg(test)]
mod tests {
    use crate::storage::*;

    use serde::{Deserialize, Serialize};
    use zeroize::Zeroize;

    #[derive(Debug, Clone, Serialize, Deserialize, Zeroize, PartialEq, Eq, PartialOrd, Ord)]
    struct TestData {
        test: String,
        hi: u64
    }

    #[test]
    fn it_works() {
        let pass = b"lolthisisapassword_asdhkggkldhashgasdljgfhasdkfjhsadfjlkasdhjfkldashgljksdfghsdfjkgand_its_pretty_long_so_cope_with_it_alright_thanks";
        let initial_data = TestData {
            test: "Lol hi whats up".to_string(),
            hi: 69420
        };

        let mut storage = SecureStorage::generate(pass, initial_data.clone()).expect("Could not generate storage");
        let raw = storage.to_raw().unwrap();

        let mut new_str = SecureStorage::<TestData>::parse(&raw).unwrap();
        new_str.try_decrypt(pass).unwrap();
        assert_eq!(&initial_data, new_str.data.as_ref().unwrap(), "Test data not the same");
        println!("{:?}", new_str);
    }
}

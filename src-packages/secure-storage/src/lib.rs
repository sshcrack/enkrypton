mod encryption;
mod storage;
mod consts;


pub use storage::*;


#[cfg(test)]
mod tests {
    use crate::storage::*;

    use serde::{Deserialize, Serialize};
    use zeroize::Zeroize;

    #[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
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

        let mut storage = SecureStorage::generate(pass, initial_data).expect("Could not generate storage");
        let raw = storage.to_raw().unwrap();

        let new_str = SecureStorage::<TestData>::parse(&raw, pass).unwrap();
        println!("{:?}", new_str);
    }
}

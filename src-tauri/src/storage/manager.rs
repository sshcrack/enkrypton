use std::path::Path;

use anyhow::Result;
use secure_storage::{Generate, SecureStorage, Parsable};
use tokio::{fs::File, io::{BufReader, AsyncReadExt, BufWriter, AsyncWriteExt}};

use super::{util::get_storage_path, StorageData};

pub type Storage = SecureStorage<StorageData>;

pub struct StorageManager {
    path: Box<Path>,
    exists: bool,
    is_valid: bool,
    is_initialized: bool,
    storage: Option<Storage>,
}

impl StorageManager {
    pub fn new() -> Self {
        let f_path = get_storage_path();
        let exists = f_path.is_file();

        Self {
            exists,
            is_valid: false,
            is_initialized: false,
            path: f_path,
            storage: None,
        }
    }

    pub async fn initialize(&mut self, pass: &str) -> Result<()> {
        let pass = pass.as_bytes();

        let storage  = if !self.exists {
            let mut storage = Storage::generate(pass, StorageData::default())?;

            let f = File::create(&self.path).await?;
            let mut writer = BufWriter::new(f);

            let raw = storage.to_raw()?;
            writer.write_all(&raw).await?;

            storage
        } else {
            let file = File::open(&self.path).await?;
            let mut reader = BufReader::new(file);

            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;

            Storage::parse(&buf, pass)?
        };

        self.storage = Some(storage);
        self.exists = true;
        self.is_initialized = true;
        self.is_valid = true;
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        return self.exists && self.is_valid;
    }
    
    pub fn is_initialized(&self) -> bool {
        return self.is_initialized;
    }
}

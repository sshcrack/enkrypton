use std::path::Path;

use anyhow::{anyhow, Result};
use log::debug;
use secure_storage::{Generate, Parsable, SecureStorage};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

use super::{util::get_storage_path, StorageData};

pub type Storage = SecureStorage<StorageData>;

pub struct StorageManager {
    path: Box<Path>,


    is_unlocked: bool,
    has_parsed: bool,

    storage: Option<Storage>,
}

impl StorageManager {
    pub fn new() -> Self {
        let f_path = get_storage_path();

        Self {
            is_unlocked: false,
            has_parsed: false,
            path: f_path,
            storage: None,
        }
    }

    /**
     * Read the storage from files or generate it with the given password
     */
    pub async fn read_or_generate(&mut self, pass: &str) -> Result<()> {
        let pass = pass.as_bytes();

        let storage = if !self.exists() {
            debug!("Generating storage...");
            let mut storage = Storage::generate(pass, StorageData::default())?;

            debug!("Writing to {:?}...", &self.path);
            let mut f = File::create(&self.path).await?;

            debug!("Getting raw...");
            let raw = storage.to_raw()?;
            debug!("Writing total of {} bytes...", raw.len());
            f.write_all(&raw).await?;

            self.is_unlocked = true;
            storage
        } else {
            let mut f = File::open(&self.path).await?;

            let mut buf = Vec::new();
            f.read_to_end(&mut buf).await?;

            Storage::parse(&buf)?
        };

        self.storage = Some(storage);
        self.has_parsed = true;
        Ok(())
    }

    pub fn verify_password(&self, pass: &[u8]) -> Result<()> {
        let storage = self.storage()?;
        storage.verify_password(pass)?;

        Ok(())
    }

    pub fn try_unlock(&mut self, pass: &[u8]) -> Result<()> {
        let storage = self.storage_mut()?;
        storage.try_decrypt(pass)?;

        self.is_unlocked = true;
        Ok(())
    }

    pub fn is_unlocked(&self) -> bool {
        return self.exists() && self.is_unlocked;
    }

    pub fn exists(&self) -> bool {
        return self.path.is_file();
    }

    pub fn has_parsed(&self) -> bool {
        return self.has_parsed;
    }

    pub fn storage(&self) -> Result<&Storage> {
        let r = self.storage.as_ref();
        if r.is_some() {
            return Ok(r.unwrap());
        }

        Err(anyhow!("Storage not initialized"))
    }

    pub fn storage_mut(&mut self) -> Result<&mut Storage> {
        let r = self.storage.as_mut();
        if r.is_some() {
            return Ok(r.unwrap());
        }

        Err(anyhow!("Storage not initialized"))
    }
}

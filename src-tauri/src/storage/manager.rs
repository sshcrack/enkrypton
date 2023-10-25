use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use anyhow::{anyhow, Result};
use log::{debug, error};
use secure_storage::{Generate, Parsable, SecureStorage};

#[cfg(target_family="unix")]
use std::fs::{Permissions, self};
#[cfg(target_family="unix")]
use smol::fs::unix::PermissionsExt;

use tokio::{
    fs::{remove_file, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
    task::{spawn, JoinHandle},
};

use super::{util::get_storage_path, StorageData};

pub type Storage = SecureStorage<StorageData>;

pub struct StorageManager {
    path: Box<Path>,

    is_unlocked: bool,
    has_parsed: bool,

    storage: Arc<RwLock<Option<Storage>>>,

    should_exit: Arc<AtomicBool>,
    dirty: Arc<AtomicBool>,

    save_thread: Option<JoinHandle<()>>,
}

impl StorageManager {
    pub fn new() -> Self {
        let f_path = get_storage_path();

        #[cfg(target_family="unix")]
        fs::set_permissions(&f_path, Permissions::from_mode(0o700)).unwrap();


        let mut e = Self {
            is_unlocked: false,
            has_parsed: false,
            path: f_path,
            storage: Arc::new(RwLock::new(None)),
            should_exit: Arc::new(AtomicBool::new(false)),
            dirty: Arc::new(AtomicBool::new(false)),
            save_thread: None,
        };

        e.run_save_thread();

        e
    }

    pub async fn exit(&mut self) -> Result<()> {
        self.should_exit.store(true, Ordering::Relaxed);
        let val = self.save_thread.take();

        if let Some(v) = val {
            v.await?;
        }

        Ok(())
    }

    // Auto-saving every 20 seconds
    fn run_save_thread(&mut self) {
        let temp = self.storage.clone();
        let dirty = self.dirty.clone();
        let path = self.path.clone();

        let should_exit = self.should_exit.clone();
        let handle = spawn(async move {
            while !should_exit.load(Ordering::Relaxed) {
                //TODO Cleanup

                thread::sleep(std::time::Duration::from_secs(20));
                if !dirty.load(Ordering::Relaxed) {
                    continue;
                }

                let mut storage = temp.write().await;
                if storage.is_none() {
                    continue;
                }

                // Copied from self.save()
                debug!("Writing to {:?}...", path);
                let f = File::create(path.clone()).await;
                if f.is_err() {
                    error!("Could not open storage file: {}", f.unwrap_err());
                    continue;
                }

                let mut f = f.unwrap();
                debug!("Getting raw...");

                let s: &mut SecureStorage<StorageData> = storage.as_mut().unwrap();
                let raw = s.to_raw();
                if raw.is_err() {
                    error!("Could not get raw storage: {}", raw.unwrap_err());
                    continue;
                }

                let raw = raw.unwrap();

                debug!("Writing total of {} bytes...", raw.len());
                let res = f.write_all(&raw).await;
                if res.is_err() {
                    error!("Could not write to storage file: {}", res.unwrap_err());
                    continue;
                }

                dirty.store(false, Ordering::Relaxed);
            }
        });

        self.save_thread = Some(handle)
    }

    /**
     * Read the storage from files or generate it with the given password
     */
    pub async fn read_or_generate(&mut self, pass: &str) -> Result<()> {
        let pass = pass.as_bytes();

        let mut newly_generated = false;
        let storage = if !self.exists() {
            debug!("Generating storage...");
            newly_generated = true;
            Storage::generate(pass, StorageData::default())?
        } else {
            let mut f = File::open(&self.path).await?;

            let mut buf = Vec::new();
            f.read_to_end(&mut buf).await?;

            Storage::parse(&buf)?
        };

        self.storage.write().await.replace(storage);
        self.has_parsed = true;

        if newly_generated {
            self.save().await?;
            self.is_unlocked = true;
        }

        Ok(())
    }

    pub async fn try_unlock(&mut self, pass: &[u8]) -> Result<()> {
        self.modify_storage(move |e| e.try_decrypt(pass)).await?;

        self.is_unlocked = true;
        Ok(())
    }

    pub async fn save(&mut self) -> Result<()> {
        debug!("Writing to {:?}...", &self.path);
        let mut f = File::create(&self.path).await?;

        debug!("Getting raw...");
        let raw = self.modify_storage(|e| e.to_raw()).await?;
        debug!("Writing total of {} bytes...", raw.len());
        f.write_all(&raw).await?;

        Ok(())
    }

    pub async fn delete(&mut self) -> Result<()> {
        if !self.exists() {
            return Ok(());
        }

        remove_file(&self.path).await?;
        self.has_parsed = false;
        self.is_unlocked = false;

        Ok(())
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub async fn data(&self) -> Option<StorageData> {
        let state = self.storage.read().await;
        if state.is_none() {
            return None;
        }

        let inner = state.as_ref().unwrap();
        return inner.data.clone();
    }

    pub async fn get_data<T, Func>(&self, f: Func) -> Result<T>
    where
        Func: FnOnce(&StorageData) -> Result<T>
    {
        let storage = self.storage.read().await;
        if let Some(s) = storage.as_ref() {
            let d = s.data.as_ref();
            if d.is_none() {
                return Err(anyhow!("Storage not initialized yet."));
            }

            let d = d.unwrap();
            return f(&d)
        }

        return Err(anyhow!("Storage not initialized yet."))
    }

    pub async fn modify_storage<Func, T>(&mut self, f: Func) -> Result<T>
    where
        Func: FnOnce(&mut Storage) -> Result<T>,
    {
        let mut storage = self.storage.write().await;

        if storage.is_none() {
            return Err(anyhow!("Storage is not initialized"));
        }

        let unwrapped = storage.as_mut().unwrap();
        let res = f(unwrapped);
        self.mark_dirty();

        Ok(res?)
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
}

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration, thread,
};

use anyhow::{anyhow, Result};
use log::{debug, error, warn};
use payloads::{
    data::StorageData, event::AppHandleExt, payloads::storage_changed::StorageChangedPayload,
};
use secure_storage::{Generate, Parsable, SecureStorage};
use shared::get_storage_path;
use shared::APP_HANDLE;

#[cfg(target_family = "unix")]
use smol::fs::unix::PermissionsExt;
#[cfg(target_family = "unix")]
use std::fs::{self, Permissions};

use tokio::{
    fs::{remove_file, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
    task::{spawn, JoinHandle},
};

/// The general type of storage that is used in this application
pub type Storage = SecureStorage<StorageData>;

/// Manages the storage file, the encryption / decryption process and saving the storage file again
pub struct StorageManager {
    /// The path to the encrypted storage file
    path: Box<Path>,

    /// whether this storage has already been unlocked
    is_unlocked: bool,
    /// whether this storage has already been parsed
    has_parsed: bool,

    /// The inner storage (this can be encrypted or decrypted, check the library for usage)
    storage: Arc<RwLock<Option<Storage>>>,

    /// whether the threads should exit
    should_exit: Arc<AtomicBool>,
    /// This is true if the storage has been modified since the last save
    dirty: Arc<AtomicBool>,

    /// The thread that is used to save to the storage
    save_thread: Option<JoinHandle<()>>,
}

impl StorageManager {
    /// Creates a new storage manager and reads the current storage file.
    /// 
    /// # Returns
    /// 
    /// The constructed storage manager
    pub fn new() -> Self {
        let f_path = get_storage_path();

        // We have to set the permission to 700 for unix to restrict access to other users / groups
        // Just the owner should be able to read this file.
        #[cfg(target_family = "unix")]
        if f_path.is_file() {
            fs::set_permissions(&f_path, Permissions::from_mode(0o700)).unwrap();
        }

        let mut e = Self {
            is_unlocked: false,
            has_parsed: false,
            path: f_path,
            storage: Arc::new(RwLock::new(None)),
            should_exit: Arc::new(AtomicBool::new(false)),
            dirty: Arc::new(AtomicBool::new(false)),
            save_thread: None,
        };

        // Starts the save thread
        e.run_save_thread();

        e
    }

    /// Tells the save thread to exit and waits for it to
    pub async fn exit(&mut self) -> Result<()> {
        self.should_exit.store(true, Ordering::Relaxed);
        let val = self.save_thread.take();

        if let Some(v) = val {
            v.await?;
        }

        Ok(())
    }

    //noinspection RsSleepInsideAsyncFunction
    /// Checks every 20 seconds if the storage is marked as dirty and if so, encrypts the data again and saves it to disk.
    fn run_save_thread(&mut self) {
        let temp = self.storage.clone();
        let dirty = self.dirty.clone();
        let path = self.path.clone();

        let should_exit = self.should_exit.clone();
        let handle = spawn(async move {
            // Checks if the current save thread should exit
            while !should_exit.load(Ordering::Relaxed) {
                //TODO Cleanup to not duplicate this func

                thread::sleep(Duration::from_secs(20));
                // Return if none of the files have been modified
                if !dirty.load(Ordering::Relaxed) {
                    continue;
                }

                let mut storage = temp.write().await;
                if storage.is_none() {
                    continue;
                }

                // Look for documentation at self.save() this is the function just copied
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


                #[cfg(target_family = "unix")]
                if path.is_file() {
                    fs::set_permissions(&path, Permissions::from_mode(0o700)).unwrap();
                }

                dirty.store(false, Ordering::Relaxed);
                debug!("Done.");
            }
        });

        self.save_thread = Some(handle)
    }

    /// Reads the storage file and decrypts it with the password, if the storage file does not exist, it will be generated
    ///
    /// # Arguments
    ///
    /// * `pass` - The password to use for decryption
    pub async fn read_or_generate(&mut self, pass: &str) -> Result<()> {
        let pass = pass.as_bytes();

        let mut newly_generated = false;
        let storage = if !self.exists()? {
            debug!("Generating storage...");
            newly_generated = true;
            // Generate a new storage with the given password
            Storage::generate(pass, StorageData::default())?
        } else {
            let mut f = File::open(&self.path).await?;

            let mut buf = Vec::new();
            f.read_to_end(&mut buf).await?;

            // Parses and decrypts the storage, fails if the wrong password has been given
            Storage::parse(&buf)?
        };

        // Save the newly parsed storage
        self.storage.write().await.replace(storage);
        self.has_parsed = true;

        if newly_generated {
            self.save().await?;
            self.is_unlocked = true;
        }

        Ok(())
    }

    /// Tries to unlock the storage with the given password, fails if the password is wrong
    ///
    /// # Arguments
    ///
    /// * `pass` - The password in binary to try to decrypt the storage with
    pub async fn try_unlock(&mut self, pass: &[u8]) -> Result<()> {
        self.modify_storage(move |e| e.try_decrypt(pass)).await?;

        self.is_unlocked = true;
        Ok(())
    }

    /// Saves the current storage to the file
    pub async fn save(&self) -> Result<()> {
        debug!("Writing to {:?}...", &self.path);

        debug!("Getting raw...");
        let raw = self.modify_storage(|e| e.to_raw()).await?;
        debug!("Writing total of {} bytes...", raw.len());
        let mut f = File::create(&self.path).await?;
        f.write_all(&raw).await?;

        #[cfg(target_family = "unix")]
        if self.path.is_file() {
            fs::set_permissions(&self.path, Permissions::from_mode(0o700)).unwrap();
        }
        Ok(())
    }

    /// Deletes the storage file (in case of a forgotten password)
    pub async fn delete(&mut self) -> Result<()> {
        if !self.path.is_file() {
            return Ok(());
        }

        remove_file(&self.path).await?;
        self.has_parsed = false;
        self.is_unlocked = false;

        Ok(())
    }

    /// Marks the storage as dirty (so it will be saved later)
    pub async fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        let res = APP_HANDLE
            .read()
            .await
            .as_ref()
            .ok_or(anyhow!("app handle not there"))
            .map(|e| e.emit_payload(StorageChangedPayload {}));

        if res.is_err() {
            warn!("Could not emit dirty event: {:?}", res.unwrap_err());
        }
    }

    /// Gets and clones the current data
    pub async fn data(&self) -> Option<StorageData> {
        let state = self.storage.read().await;
        if state.is_none() {
            return None;
        }

        let inner = state.as_ref().unwrap();
        inner.data.clone()
    }

    /// Gets the data of the current storage if there is some and passes it to the given function
    ///
    /// # Arguments
    ///
    /// * `f` - The function to process the data
    pub async fn get_data<T, Func>(&self, f: Func) -> Result<T>
    where
        Func: FnOnce(&StorageData) -> Result<T>,
    {
        let storage = self.storage.read().await;
        if let Some(s) = storage.as_ref() {
            let d = s.data.as_ref();
            if d.is_none() {
                return Err(anyhow!("Storage not initialized yet."));
            }

            let d = d.unwrap();
            return f(&d);
        }

        Err(anyhow!("Storage not initialized yet."))
    }

    /// Modifies the storage with the given function
    ///
    /// # Arguments
    ///
    /// * `f` - The function to modify the storage with.
    /// 
    /// # Returns
    /// 
    /// Returns the result of the function `f`
    pub async fn modify_storage<Func, T>(&self, f: Func) -> Result<T>
    where
        Func: FnOnce(&mut Storage) -> Result<T>,
    {
        let mut storage = self.storage.write().await;

        if storage.is_none() {
            return Err(anyhow!("Storage is not initialized"));
        }

        let unwrapped = storage.as_mut().unwrap();
        let res = f(unwrapped);
        self.mark_dirty().await;

        Ok(res?)
    }

    /// This is just a wrapper function around `modify_storage` that passes just the data to the function
    ///
    /// # Arguments
    ///
    /// * `f` - The function to run
    ///
    /// # Returns
    ///
    /// The result of the function that ran
    pub async fn modify_storage_data<Func, T>(&self, f: Func) -> Result<T>
    where
        Func: FnOnce(&mut StorageData) -> Result<T>,
    {
        self.modify_storage(|e| {
            if let Some(data) = e.data.as_mut() {
                return f(data);
            }

            return Err(anyhow!("Storage not initialized"));
        })
        .await
    }

    /// Checks if the storage is already unlocked
    ///
    /// # Returns
    ///
    /// Returns whether the storage is unlocked
    pub fn is_unlocked(&self) -> Result<bool> {
        Ok(self.exists()? && self.is_unlocked)
    }

    /// Checks if the storage file exists
    ///
    /// # Returns
    ///
    /// Returns whether the storage file exists
    pub fn exists(&self) -> Result<bool> {
        Ok(self.path.is_file() && self.path.metadata()?.len() != 0)
    }

    /// Checks if the storage has already been parsed
    ///
    /// # Returns
    ///
    /// Returns whether the storage has already been parsed
    pub fn has_parsed(&self) -> bool {
        self.has_parsed
    }
}

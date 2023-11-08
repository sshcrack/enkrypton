use std::sync::Arc;

use lazy_static::lazy_static;


pub mod helpers;
mod data;
mod manager;

pub use manager::*;
pub use data::*;
use tokio::sync::RwLock;

lazy_static! {
    /// Holds the whole storage data (keys, values, etc.)
    pub static ref STORAGE: Arc<RwLock<StorageManager>> = Arc::new(RwLock::new(StorageManager::new()));
}
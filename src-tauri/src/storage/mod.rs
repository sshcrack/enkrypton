use std::sync::Arc;

use lazy_static::lazy_static;


mod util;
pub mod helpers;
mod data;
mod manager;
pub mod encryption;

pub use manager::*;
pub use data::*;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref STORAGE: Arc<RwLock<StorageManager>> = Arc::new(RwLock::new(StorageManager::new()));
}
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Only one Storage instance is allowed.
#[derive(Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct StorageData {
    #[zeroize(skip)]
    nicknames: HashMap<String, String>,
    messages: Vec<Chat>,
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            nicknames: HashMap::new(),
            messages: Vec::new(),
        }
    }
}

// Only one Storage instance is allowed.
#[derive(Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct Chat {}

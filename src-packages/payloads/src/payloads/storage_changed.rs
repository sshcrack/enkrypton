use serde::{Serialize, Deserialize};

use crate::event::Sendable;

/// Tells the frontend to update after the storage has changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChangedPayload {}

impl Sendable for StorageChangedPayload {
    fn get_name(&self) -> String {
        "storage_changed".to_string()
    }
}
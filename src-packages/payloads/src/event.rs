use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Error};

//noinspection SpellCheckingInspection
/// Describes any sendable payload and contains a function to get the name of the payload
pub trait Sendable {
    /// The name of the payload that should be used for tauri
    fn get_name(&self) -> String;
}

/// A trait to extend the AppHandle with a function to send a payload
/// Extension trait for the `AppHandle` type.
pub trait AppHandleExt {
    /// Emits a payload of type `T`.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload to be emitted.
    ///
    fn emit_payload<'a, T: Sendable + Serialize + Deserialize<'a> + Clone>(&self, payload: T) -> Result<(), Error>;
}

impl AppHandleExt for AppHandle {
    fn emit_payload<'a, T: Sendable + Serialize + Deserialize<'a> + Clone>(&self, payload: T) -> Result<(), Error> {
        self.emit_all(&payload.get_name(), payload)
    }
}
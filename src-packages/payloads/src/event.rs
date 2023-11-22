use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Error};

pub trait Sendable {
    fn get_name(&self) -> String;
}

pub trait AppHandleExt {
    fn emit_payload<'a, T: Sendable + Serialize + Deserialize<'a> + Clone>(&self, payload: T) -> Result<(), Error>;
}

impl AppHandleExt for AppHandle {
    fn emit_payload<'a, T: Sendable + Serialize + Deserialize<'a> + Clone>(&self, payload: T) -> Result<(), Error> {
        self.emit_all(&payload.get_name(), payload)
    }
}
use std::sync::Arc;
use tokio::sync::RwLock;

use tauri::AppHandle;
use lazy_static::lazy_static;

lazy_static! {
    /// An App-Handle to the application (which basically holds every windows, can send events etc.)
    pub static ref APP_HANDLE: Arc<RwLock<Option<AppHandle>>> = RwLock::new(None).into();

    /// The message to send when enkrypton is up and running (at path /)
    pub static ref DEFAULT_HTTP_RETURN: String = "Hi, yes I'm connected!".to_string();
}


/// Gets the app handle of the application, panics if it's not there
///
/// # Returns
///
/// The current app handle tauri has launched
pub async fn get_app() -> AppHandle {
    let state = APP_HANDLE.read().await;
    let handle = state.as_ref().unwrap();

    handle.clone()
}


mod directories;
pub use directories::*;
pub mod util;
pub mod config;
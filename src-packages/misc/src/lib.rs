use lazy_static::lazy_static;

mod traits;
mod manager;
mod connection;

pub use connection::*;
pub use manager::*;
pub use traits::*;

lazy_static! {
    /// An App-Handle to the application (which basically holds every windows, can send events etc.)
    pub static ref APP_HANDLE: Arc<RwLock<Option<AppHandle>>> = RwLock::new(None).into();

}
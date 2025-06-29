use std::{path::PathBuf, sync::Arc, thread::JoinHandle};

use async_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use shared::get_tor_path;
use tokio::sync::RwLock;

use super::misc::messages::{Client2TorMsg, Tor2ClientMsg};
#[cfg(feature = "snowflake")]
use std::path::Path;

lazy_static! {
    /// Hash of the Tor binary, used to verify the integrity of the binary
    pub static ref TOR_BINARY_HASH: String = get_tor_binary_hash();

    /// The actual path to the binary
    pub static ref TOR_BINARY_PATH: PathBuf = get_tor_path();

    /// This thread spawns and handles the Tor process.
    pub static ref TOR_THREAD: Arc<RwLock<Option<JoinHandle<()>>>> = Arc::default();
    /// Basic lock to prevent multiple tor instances from starting (read is blocked until start is done)
    pub static ref TOR_START_LOCK: Arc<RwLock<bool>> = Arc::default();

    /// Messages that should be send to the tor handling thread. For now this is just to handle exit events
    pub static ref TO_TOR_TX: Arc<RwLock<Option<Sender<Client2TorMsg>>>> = Arc::default();
    /// Internal receiver for the tor handle
    pub(super) static ref TO_TOR_RX: Arc<RwLock<Option<Receiver<Client2TorMsg>>>> = Arc::default();

    /// All Kinds of initialization messages, ErrorMessages etc. from the tor thread / handle
    pub static ref FROM_TOR_RX: Arc<RwLock<Option<Receiver<Tor2ClientMsg>>>> = Arc::default();
    /// Again, internal stuff to actually send these messages inside of the tor handle
    pub(super) static ref FROM_TOR_TX: Arc<RwLock<Option<Sender<Tor2ClientMsg>>>> = Arc::default();

    /// Keep 20 log messages in memory
    pub(super) static ref MAX_LOG_SIZE: usize = 20;

}

#[cfg(feature="snowflake")]
lazy_static! {
    /// Hash of the Snowflake binary, used to verify the integrity of the binary
    pub static ref SNOWFLAKE_BINARY_HASH: String = get_snowflake_binary_hash();
}

/// Initializes every channel used to communicate with the tor thread
pub async fn setup_tor_channels() {
    let (to_tx, to_rx) = async_channel::unbounded::<Client2TorMsg>();
    let (from_tx, from_rx) = async_channel::unbounded::<Tor2ClientMsg>();

    TO_TOR_TX.write().await.replace(to_tx);
    TO_TOR_RX.write().await.replace(to_rx);

    FROM_TOR_TX.write().await.replace(from_tx);
    FROM_TOR_RX.write().await.replace(from_rx);
}

/// Gets the hash of the snowflake binary, which is platform specific so this is just a helper function
///
/// # Returns
///
/// The hash of the snowflake binary encoded in hex
fn get_tor_binary_hash() -> String {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/windows/x86_64/tor.hash"));

    #[cfg(all(
        target_os = "windows",
        target_arch = "x86",
        not(target_arch = "x86_64")
    ))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/windows/i686/tor.hash"));

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/linux/x86_64/tor.hash"));

    #[cfg(all(target_os = "linux", target_arch = "x86", not(target_arch = "x86_64")))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/windows/i686/tor.hash"));

    // Checks if the hash is valid
    hex::decode(hash).unwrap();

    String::from(hash)
}


/// Gets the hash of the tor binary, which is platform specific so this is just a helper function
///
/// # Returns
///
/// The hash of the tor binary encoded in hex
#[cfg(feature="snowflake")]
fn get_snowflake_binary_hash() -> String {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/windows/x86_64/snowflake-client.hash"));

    #[cfg(all(
        target_os = "windows",
        target_arch = "x86",
        not(target_arch = "x86_64")
    ))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/windows/i686/snowflake-client.hash"));

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/linux/x86_64/snowflake-client.hash"));

    #[cfg(all(target_os = "linux", target_arch = "x86", not(target_arch = "x86_64")))]
    let hash = include_str!(concat!(env!("OUT_DIR"), "/windows/i686/snowflake-client.hash"));

    // Checks if the hash is valid
    hex::decode(hash).unwrap();

    String::from(hash)
}


#[cfg(feature = "snowflake")]
/// Returns the absolute path of the pluggable transport path used in tor (contains pt_config.json for example)
pub fn get_pluggable_transport() -> Box<Path> {
    TOR_BINARY_PATH
        .parent()
        .unwrap()
        .join("pluggable_transports")
        .into_boxed_path()
}

#[cfg(feature="snowflake")]
/// Gets the absolute path of the snowflake binary
pub fn get_snowflake_path() -> Box<Path> {
    let path = get_pluggable_transport();
    let rel = get_rel_snowflake();

    let mut path = path.join("..");
    path.push(rel);

    path.into_boxed_path()
}

#[cfg(feature = "snowflake")]
/// Gets the relative path of the snowflake binary
pub fn get_rel_snowflake() -> String {
    #[cfg(target_os = "windows")]
    let snowflake_bin = "snowflake-client.exe";
    #[cfg(not(target_os = "windows"))]
    let snowflake_bin = "snowflake-client";

    let parent_dir = get_pluggable_transport();
    let parent_dir = parent_dir.file_name().unwrap().to_string_lossy();

    format!("{}/{}", parent_dir, snowflake_bin)
}

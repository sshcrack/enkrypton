use std::{path::PathBuf, sync::Arc, thread::JoinHandle};

use async_channel::{Receiver, Sender};
use lazy_static::lazy_static;

use openssl::{hash::MessageDigest, rsa::Padding};
use tauri::{async_runtime::RwLock, AppHandle};

use crate::util::get_root_dir;

use super::misc::messages::{Client2TorMsg, Tor2ClientMsg};

lazy_static! {
    /// An App-Handle to the application (which basically holds every windows, can send events etc.)
    pub static ref APP_HANDLE: Arc<RwLock<Option<AppHandle>>> = RwLock::new(None).into();



    /// The message to send when enkrypton is up and running (at path /)
    pub static ref DEFAULT_HTTP_RETURN: String = "Hi, yes I'm connected!".to_string();

    /// This is the path to the tor binary in zip
    pub static ref TOR_ZIP_PATH: PathBuf = get_tor_zip_path();

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

    /// Default hash used for signing / verifying
    pub static ref DIGEST: MessageDigest = MessageDigest::md5();
    /// Default Padding for RSA encryption
    pub static ref RSA_PADDING: Padding = Padding::PKCS1;
}

/// Sets all channels up. Documentation is at the channels themselves
pub async fn setup_channels() {
    let (to_tx, to_rx) = async_channel::unbounded::<Client2TorMsg>();
    let (from_tx, from_rx) = async_channel::unbounded::<Tor2ClientMsg>();

    TO_TOR_TX.write().await.replace(to_tx);
    TO_TOR_RX.write().await.replace(to_rx);

    FROM_TOR_TX.write().await.replace(from_tx);
    FROM_TOR_RX.write().await.replace(from_rx);
}


/// Gets the hash of the tor binary, which is platform specific so this is just a helper function
/// # Returns
/// The hash of the tor binary encoded in hex
fn get_tor_binary_hash() -> String {
    #[cfg(all(target_os ="windows", target_arch = "x86_64"))]
    let hash = include_str!("../assets/windows/x86_64/tor.exe.hash");

    #[cfg(all(target_os ="windows", target_arch = "x86", not(target_arch="x86_64")))]
    let hash = include_str!("../assets/windows/i686/tor.exe.hash");

    #[cfg(all(target_os ="linux", target_arch = "x86_64"))]
    let hash = include_str!("../assets/linux/x86_64/tor.hash");

    #[cfg(all(target_os ="linux", target_arch = "x86", not(target_arch="x86_64")))]
    let hash = include_str!("../assets/windows/i686/tor.hash");

    // Checks if the hash is valid
    hex::decode(hash).unwrap();

    return String::from(hash);
}

/// The file to tell tor what to do (https://manpages.debian.org/jessie/tor/torrc.5)
pub fn get_torrc() -> PathBuf {
    let mut dir = get_root_dir();
    dir.push("torrc");

    return dir;
}
/// Path to the tor binary which is still zipped up
fn get_tor_zip_path() -> PathBuf {
    let tor_write_path = get_root_dir();

    return tor_write_path.join("enkrypton_tor.zip");
}

/// Path to the extracted tor binary, again this is platform specific
fn get_tor_path() -> PathBuf {
    let tor_write_path = get_root_dir();
    #[cfg(target_os="windows")]
    return tor_write_path.join("tor.exe");

    #[cfg(target_os="linux")]
    return tor_write_path.join("tor");
}
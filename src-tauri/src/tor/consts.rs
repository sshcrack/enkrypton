use std::{path::PathBuf, sync::Arc, thread::JoinHandle};

use async_channel::{Receiver, Sender};
use lazy_static::lazy_static;

use tauri::{async_runtime::RwLock, AppHandle};

use crate::util::get_root_dir;

use super::misc::messages::{Client2TorMsg, Tor2ClientMsg};

lazy_static! {
    pub static ref APP_HANDLE: Arc<RwLock<Option<AppHandle>>> = RwLock::new(None).into();


    pub static ref DEFAULT_HTTP_RETURN: String = "Hi, yes I'm connected!".to_string();

    pub static ref TOR_ZIP_PATH: PathBuf = get_tor_zip_path();

    pub static ref TOR_BINARY_HASH: String = get_tor_binary_hash();
    pub static ref TOR_BINARY_PATH: PathBuf = get_tor_path();

    pub static ref TOR_THREAD: Arc<RwLock<Option<JoinHandle<()>>>> = Arc::default();
    pub static ref TOR_START_LOCK: Arc<RwLock<bool>> = Arc::default();

    pub static ref TO_TOR_TX: Arc<RwLock<Option<Sender<Client2TorMsg>>>> = Arc::default();
    pub(super) static ref TO_TOR_RX: Arc<RwLock<Option<Receiver<Client2TorMsg>>>> = Arc::default();

    pub static ref FROM_TOR_RX: Arc<RwLock<Option<Receiver<Tor2ClientMsg>>>> = Arc::default();
    pub(super) static ref FROM_TOR_TX: Arc<RwLock<Option<Sender<Tor2ClientMsg>>>> = Arc::default();

    /* In total 20 log messages to keep in memory */
    pub(super) static ref MAX_LOG_SIZE: usize = 20;
}

pub async fn setup_channels() {
    let (to_tx, to_rx) = async_channel::unbounded::<Client2TorMsg>();
    let (from_tx, from_rx) = async_channel::unbounded::<Tor2ClientMsg>();

    TO_TOR_TX.write().await.replace(to_tx);
    TO_TOR_RX.write().await.replace(to_rx);

    FROM_TOR_TX.write().await.replace(from_tx);
    FROM_TOR_RX.write().await.replace(from_rx);
}


fn get_tor_binary_hash() -> String {
    #[cfg(all(target_os ="windows", target_arch = "x86_64"))]
    let hash = include_str!("../assets/windows/x86_64/tor.exe.hash");

    #[cfg(all(target_os ="windows", target_arch = "x86", not(target_arch="x86_64")))]
    let hash = include_str!("../assets/windows/i686/tor.exe.hash");

    #[cfg(all(target_os ="linux", target_arch = "x86_64"))]
    let hash = include_str!("../assets/linux/x86_64/tor.hash");

    #[cfg(all(target_os ="linux", target_arch = "x86", not(target_arch="x86_64")))]
    let hash = include_str!("../assets/windows/i686/tor.hash");

    hex::decode(hash).unwrap();

    return String::from(hash);
}

pub fn get_torrc() -> PathBuf {
    let mut dir = get_root_dir();
    dir.push("torrc");

    return dir;
}

fn get_tor_zip_path() -> PathBuf {
    let tor_write_path = get_root_dir();

    return tor_write_path.join("enkrypton_tor.zip");
}

fn get_tor_path() -> PathBuf {
    let tor_write_path = get_root_dir();
    #[cfg(target_os="windows")]
    return tor_write_path.join("tor.exe");

    #[cfg(target_os="linux")]
    return tor_write_path.join("tor");
}
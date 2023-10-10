use std::{env::current_exe, path::PathBuf, sync::Arc, thread::JoinHandle};

use async_channel::{Receiver, Sender};
use lazy_static::lazy_static;

use tauri::{async_runtime::RwLock, AppHandle};

use super::misc::messages::{Client2TorMsg, Tor2ClientMsg};

lazy_static! {
    pub static ref APP_HANDLE: Arc<RwLock<Option<AppHandle>>> = RwLock::new(None).into();


    pub static ref DEFAULT_HTTP_RETURN: String = "Hi, yes I'm connected!".to_string();

    pub static ref TOR_BINARY_HASH: String = get_tor_hash();
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

fn get_tor_hash() -> String {
    let hash = include_str!("../assets/tor.exe.hash");
    hex::decode(hash).unwrap();

    return String::from(hash);
}

pub fn get_root_dir() -> PathBuf {
    let mut buf = current_exe().unwrap().parent().unwrap().to_path_buf();
    buf.push("enkrypton");

    buf
}

pub fn get_torrc() -> PathBuf {
    let mut dir = get_root_dir();
    dir.push("torrc");

    return dir;
}

fn get_tor_path() -> PathBuf {
    let mut tor_write_path = get_root_dir();
    tor_write_path.set_file_name("enkrypton_tor.exe");

    return tor_write_path;
}

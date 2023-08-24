use std::{env::current_exe, path::PathBuf, thread::JoinHandle, sync::Arc};

use async_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use tauri::async_runtime::RwLock;

use super::manager::{Tor2ClientMsg, Client2TorMsg};

// https://check.torproject.org/api/ip
lazy_static! {
    pub static ref TOR_BINARY_HASH: String = get_tor_hash();
    pub static ref TOR_BINARY_PATH: PathBuf = get_tor_path();
    pub static ref TOR_THREAD: Arc<RwLock<Option<JoinHandle<()>>>> = Arc::default();

    pub static ref TO_TOR_TX: Arc<RwLock<Option<Sender<Client2TorMsg>>>> = Arc::default();
    pub static ref FROM_TOR_RX: Arc<RwLock<Option<Receiver<Tor2ClientMsg>>>> = Arc::default();

}

fn get_tor_hash() -> String {
    let hash = include_str!("../assets/tor.exe.hash");
    hex::decode(hash).unwrap();

    return String::from(hash);
}


fn get_tor_path() -> PathBuf {
    let mut tor_write_path = current_exe().unwrap();
    tor_write_path.set_file_name("tor_proxy.exe");

    return tor_write_path;
}
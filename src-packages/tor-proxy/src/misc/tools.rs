use async_channel::{Receiver, Sender};

use crate::consts::*;

use super::messages::{Client2TorMsg, Tor2ClientMsg};

pub async fn get_to_tor_tx() -> Sender<Client2TorMsg> {
    return TO_TOR_TX.read().await.clone().unwrap();
}
pub(in crate) async fn get_to_tor_rx() -> Receiver<Client2TorMsg> {
    return TO_TOR_RX.read().await.clone().unwrap();
}

pub(in crate) async fn get_from_tor_tx() -> Sender<Tor2ClientMsg> {
    return FROM_TOR_TX.read().await.clone().unwrap();
}
pub async fn get_from_tor_rx() -> Receiver<Tor2ClientMsg> {
    return FROM_TOR_RX.read().await.clone().unwrap();
}

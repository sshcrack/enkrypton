use async_channel::{Receiver, Sender};

use crate::consts::*;

use super::messages::{Client2TorMsg, Tor2ClientMsg};

/// The channel used to send messages to the tor proxy such as Exit Messages
pub async fn get_to_tor_tx() -> Sender<Client2TorMsg> {
    return TO_TOR_TX.read().await.clone().unwrap();
}
/// The channel used to receive messages from the tor proxy such as Exit Messages
pub(in crate) async fn get_to_tor_rx() -> Receiver<Client2TorMsg> {
    return TO_TOR_RX.read().await.clone().unwrap();
}

/// The channel used to send messages from the tor mainloop to the backend client
pub(in crate) async fn get_from_tor_tx() -> Sender<Tor2ClientMsg> {
    return FROM_TOR_TX.read().await.clone().unwrap();
}
/// Used to receive the messages above
pub async fn get_from_tor_rx() -> Receiver<Tor2ClientMsg> {
    return FROM_TOR_RX.read().await.clone().unwrap();
}

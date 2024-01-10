use async_channel::{Receiver, Sender};

use crate::consts::*;

use super::messages::{Client2TorMsg, Tor2ClientMsg};

/// The channel used to send messages to the tor proxy such as Exit Messages
/// 
/// # Returns
/// A sender to send messages to the tor proxy (like exit messages)
pub async fn get_to_tor_tx() -> Sender<Client2TorMsg> {
    return TO_TOR_TX.read().await.clone().unwrap();
}

/// The channel used to receive messages from the tor proxy such as Exit Messages.
/// Read from in the tor mainloop
/// 
/// # Returns
/// A receiver to receive messages from the tor proxy again, something like exit messages
pub(in crate) async fn get_to_tor_rx() -> Receiver<Client2TorMsg> {
    return TO_TOR_RX.read().await.clone().unwrap();
}

/// The channel used to send messages from the tor mainloop to the backend client
/// (like stdout, stderr and other status messages).
/// Used by the tor mainloop
/// 
/// # Returns
/// 
/// A sender to send messages to the backend client
pub(in crate) async fn get_from_tor_tx() -> Sender<Tor2ClientMsg> {
    return FROM_TOR_TX.read().await.clone().unwrap();
}

/// Used to receive incoming messages from the tor proxy process (like stdout, stderr etc.)
/// 
/// # Returns
/// 
/// A receiver to receive messages from the tor proxy
pub async fn get_from_tor_rx() -> Receiver<Tor2ClientMsg> {
    return FROM_TOR_RX.read().await.clone().unwrap();
}

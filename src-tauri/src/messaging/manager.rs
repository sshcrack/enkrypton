use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{client::MessagingClient, webserver::ws_manager::MessagingServer};

pub enum Role {
    Client2Server(MessagingClient),
    Server2Client(MessagingServer)
}

pub struct MessagingManager {
    connections: HashMap<String, Role>
}

lazy_static! {
    pub static ref MESSAGING: MessagingManager = MessagingManager::new();
}

impl MessagingManager {
    fn new() -> Self {
        MessagingManager {
            connections: HashMap::new()
        }
    }

    fn connect(&mut self, onion_hostname: String) {
    }
}
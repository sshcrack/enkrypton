use std::collections::HashMap;

pub enum Role {
    Client2Server,
    Server2Client
}

pub struct MessagingManager {
    connections: HashMap<String, Role>
}

impl MessagingManager {
    pub fn new() -> Self {
        MessagingManager {
            connections: HashMap::new()
        }
    }
}
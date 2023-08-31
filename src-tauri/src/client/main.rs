use anyhow::Result;

#[derive(Debug)]
pub struct MessagingClient {}

impl MessagingClient {
    pub async fn new(_onion_addr: &str) -> Result<Self> {

        return Ok(MessagingClient {});
    }
}

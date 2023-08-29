use std::collections::HashMap;

use anyhow::{anyhow, Result};
use url::Url;

use super::Client;

struct Request {
    client: Client,
    url: Url,
    headers: HashMap<String, String>,
}

impl Request {
    pub(super) fn from_client(client: Client, url: Url) -> Self {
        Self {
            client,
            headers: HashMap::new(),
            url,
        }
    }

    fn header(mut self, header: &str, value: &str) -> Self {
        self.headers.insert(header.to_string(), value.to_string());

        self
    }

    fn get_headers(&self) -> String {
        let lines = Vec::<String>::new();
        for (key, value) in self.headers.iter() {
            lines.push(format!("{}: {}", key, value))
        }

        return lines.join("\r\n");
    }

    /**
     * TODO (maybe?) support other protocols than https
     */
    fn send(self) -> Result<String> {
        let port = self.url.port_or_known_default().ok_or(anyhow!(
            "Could not get standard port of url {}",
            self.url.to_string()
        ))?;

        let server_name = self
            .url
            .host_str()
            .ok_or(anyhow!("Url has to have a host."))?;

        let server_with_port = format!("{}:{}", server_name, port);
    }
}

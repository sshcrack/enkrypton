use std::collections::HashMap;

use anyhow::{anyhow, Result};
use smol::io::AsyncWriteExt;
use url::Url;

use crate::util::get_servername;

use super::{response::Response, Client};

pub struct Request<'a> {
    client: &'a Client,
    url: String,
    method: String,
    headers: HashMap<String, String>,
}

impl<'a> Request<'a> {
    pub(super) fn from_client(client: &'a Client, method: &str, url: &str) -> Self {
        Self {
            client,
            headers: HashMap::new(),
            //REVIEW - Maybe add other methods later?
            method: method.to_string(),
            url: url.to_string(),
        }
    }

    pub fn header(mut self, header: &str, value: &str) -> Self {
        self.headers.insert(header.to_string(), value.to_string());

        self
    }

    fn get_headers(&self, host: &str) -> String {
        let mut lines = Vec::<String>::new();
        lines.push(format!("Host: {}", host));
        for (key, value) in self.headers.iter() {
            lines.push(format!("{}: {}", key, value))
        }

        return lines.join("\r\n");
    }

    fn get_full_headers(&self, url: &Url, path: &str) -> Result<String> {
        let start = format!("{} {} HTTP/1.1\r\n", self.method, path);
        let server_name_raw = url.host_str().ok_or(anyhow!("Url has to have a host."))?;
        let headers = self.get_headers(server_name_raw);

        return Ok(format!("{}{}\r\n\r\n", start, headers));
    }

    /**
     * TODO (maybe?) support other protocols than https
     */
    pub async fn send(self) -> Result<Response> {
        let url = Url::parse(&self.url)?;
        let port = url
            .port_or_known_default()
            .ok_or(anyhow!("Could not get standard port of url {}", self.url))?;

        let mut path = url.path();
        if path.is_empty() {
            path = "/";
        }

        let proxy_conn = self.client.proxy().connect(&url).await?;
        let mut stream = self.client.create_connection(proxy_conn, &url).await?;

        let prepend = self.get_full_headers(&url, path)?;

        println!("Writing headers \n----\n{}\n----", prepend);
        stream.write_all(prepend.as_bytes()).await?;

        let resp = Response::from_stream(stream).await?;
        return Ok(resp);
    }
}

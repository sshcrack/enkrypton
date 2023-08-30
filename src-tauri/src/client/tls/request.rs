use std::{
    collections::HashMap,
    io::{Read, Write},
};

use anyhow::{anyhow, Result};
use rustls::Stream;
use tauri::async_runtime::block_on;
use url::Url;

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

    fn get_headers(&self) -> String {
        let lines = Vec::<String>::new();
        for (key, value) in self.headers.iter() {
            lines.push(format!("{}: {}", key, value))
        }

        return lines.join("\r\n");
    }

    fn get_full_headers(&self, path: &str) -> String {
        let start = format!("{} {} HTTP/1.1\r\n", self.method, path);
        let headers = self.get_headers();

        return format!("{}\r\n{}\r\n\r\n", start, headers);
    }

    /**
     * TODO (maybe?) support other protocols than https
     * ATTENTION: Blocking!!
     */
    pub fn send(self) -> Result<Response> {
        let url = Url::parse(&self.url)?;
        let port = url.port_or_known_default().ok_or(anyhow!(
            "Could not get standard port of url {}",
            self.url
        ))?;

        let server_name = url
            .host_str()
            .ok_or(anyhow!("Url has to have a host."))?;

        let server_with_port = format!("{}:{}", server_name, port);
        let mut path = url.path();
        if path.is_empty() {
            path = "/";
        }

        let conn_future = self.client.proxy.connect(&server_with_port);
        let proxy_conn = block_on(conn_future)?;
        let c_conn = self.client.create_connection(server_name)?;

        let stream = Stream::new(&mut c_conn, &mut proxy_conn);

        let prepend = self.get_full_headers(path);
        stream.write_all(prepend.as_bytes());

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).unwrap();

        let resp = Response::from_bytes(buffer);

        return Ok(resp);
    }
}

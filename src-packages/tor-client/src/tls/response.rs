use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_rustls::client::TlsStream;
use log::{debug, warn};
use serde::de::DeserializeOwned;

use smol::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

pub struct Response {
    reader: BufReader<TlsStream<TcpStream>>,
    status_code: Option<u32>,
    headers: Option<HashMap<String, String>>,
}

const HEADER_SEPARATOR: &str = ": ";
impl Response {
    pub async fn headers(&mut self) -> Result<HashMap<String, String>> {
        if let Some(headers) = self.headers.as_ref() {
            return Ok(headers.clone());
        }

        if self.status_code.is_none() {
            self.read_status().await?;
        }

        let res = self.read_headers().await?;
        Ok(res)
    }

    async fn read_status(&mut self) -> Result<u32> {
        let mut buf = String::new();

        self.reader.read_line(&mut buf).await?;
        let mut split = buf.split_ascii_whitespace();

        let protocol = split.next().ok_or(anyhow!("Not starting with protocol"))?;
        let status_code = split.next().ok_or(anyhow!("No status code"))?;
        let status_code: u32 = status_code.parse()?;

        if !protocol.contains("HTTP") {
            return Err(anyhow!("Invalid protocol {}", protocol));
        }

        self.status_code = Some(status_code);
        Ok(status_code)
    }

    pub async fn status(&mut self) -> Result<u32> {
        if let Some(code) = self.status_code {
            return Ok(code);
        }

        let res = self.read_status().await?;
        self.status_code = Some(res);

        return Ok(res);
    }

    async fn read_headers(&mut self) -> Result<HashMap<String, String>> {
        let mut buf = String::new();

        let mut headers = HashMap::<String, String>::new();
        loop {
            buf.clear();
            let res = self.reader.read_line(&mut buf).await;
            if res.is_err() || res.unwrap() == 0 {
                break;
            }

            let buf = buf.replace("\r", "");
            debug!("Buf: {}", buf);

            let only_whitespace = buf.chars().all(|e| e.is_whitespace() || e == '\n');

            if only_whitespace {
                break;
            }

            let mut pair = Vec::from_iter(buf.split(HEADER_SEPARATOR));
            if pair.len() < 2 {
                let char_vec: Vec<String> =
                    buf.chars().map(|e| e.escape_debug().to_string()).collect();

                warn!(
                    "Server returned invalid header '{}' (chars: {:?})",
                    buf, char_vec
                );
                continue;
            }

            let key = pair.remove(0).to_string();
            let value = pair
                .join(HEADER_SEPARATOR)
                .trim_end_matches(|e| e == '\n' || e == '\r')
                .to_string();

            headers.insert(key, value);
        }

        debug!("Headers are {:#?}", headers);

        self.headers.replace(headers.clone());
        Ok(headers)
    }

    pub(super) async fn from_stream(stream: TlsStream<TcpStream>) -> Result<Self> {
        let reader = BufReader::new(stream);

        Ok(Response {
            reader,
            status_code: None,
            headers: None,
        })
    }

    pub async fn json<'a, T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let txt = self.text().await?;

        let json = serde_json::from_str::<T>(&txt)?;
        Ok(json)
    }

    pub async fn text(mut self) -> Result<String> {
        let mut buf = String::new();
        self.reader.read_to_string(&mut buf).await?;

        return Ok(buf);
    }
}

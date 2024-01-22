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

/// The seperator between the header key and value
const HEADER_SEPARATOR: &str = ": ";

/// Represents a response from the web client.
impl Response {
    /// Retrieves the headers of the response.
    ///
    /// If the headers have already been retrieved, it returns a clone of the cached headers.
    /// Otherwise, it reads the status code and then reads the headers from the stream.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` of the headers.
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

    /// Reads the status code from the stream and stores it in `self`.
    /// Reads from the incoming buffer
    ///
    /// # Returns
    ///
    /// The status code of this response
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

    /// Reads the status code and stores it in `self`
    /// of the response or returns the cached status code.
    ///
    /// # Returns
    ///
    /// A `Result` containing the status code as a `u32`
    pub async fn status(&mut self) -> Result<u32> {
        if let Some(code) = self.status_code {
            return Ok(code);
        }

        let res = self.read_status().await?;
        self.status_code = Some(res);

        return Ok(res);
    }

    /// Reads the headers from the incoming stream, parses them and stores them in a hash map.
    ///
    /// # Returns
    ///
    /// A hashmap of the headers
    async fn read_headers(&mut self) -> Result<HashMap<String, String>> {
        let mut buf = String::new();

        // The hashmap used to store the headers by its key
        let mut headers = HashMap::<String, String>::new();
        loop {
            // Clearing the string buffer
            buf.clear();

            // Reads a line from the stream
            let res = self.reader.read_line(&mut buf).await;
            if res.is_err() || res.unwrap() == 0 {
                break;
            }

            // Replace weird windows line endings
            let buf = buf.replace("\r", "");
            debug!("Buf: {}", buf);

            // If the line is empty, we are done
            let only_whitespace = buf.chars().all(|e| e.is_whitespace() || e == '\n');

            if only_whitespace {
                break;
            }

            // Split the line at the first occurrence of the separator
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

            // And extracting the key and value
            let key = pair.remove(0).to_string();
            let value = pair
                .join(HEADER_SEPARATOR)
                .trim_end_matches(|e| e == '\n' || e == '\r')
                .to_string();

            headers.insert(key, value);
        }

        debug!("Headers are {:#?}", headers);

        // Done, storing the headers in self
        self.headers.replace(headers.clone());
        Ok(headers)
    }

    /// Creates a `Response` from the given TLS stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TLS stream to read from.
    ///
    /// # Returns
    ///
    /// Returns the created `Response` 
    pub(super) async fn from_stream(stream: TlsStream<TcpStream>) -> Self {
        let reader = BufReader::new(stream);

        Response {
            reader,
            status_code: None,
            headers: None,
        }
    }

    /// Reads the body of the response and deserializing it to json
    /// 
    /// # Returns
    /// 
    /// The deserialized json object
    pub async fn json<'a, T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let txt = self.text().await?;

        let json = serde_json::from_str::<T>(&txt)?;
        Ok(json)
    }

    /// Reads the body of the response and returns it as a string
    /// 
    /// # Returns
    /// 
    /// The response body as string
    pub async fn text(mut self) -> Result<String> {
        let mut buf = String::new();
        self.reader.read_to_string(&mut buf).await?;

        return Ok(buf);
    }
}

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use smol::io::AsyncWriteExt;
use url::Url;

use super::{response::Response, WebClient};

/// Represents a request to a server which should be made
pub struct Request<'a> {
    /// The client to use for this request
    client: &'a WebClient,
    /// The url to request
    url: String,
    /// Method to use for this request
    method: String,
    /// Headers to send with this request
    headers: HashMap<String, String>,
}

impl<'a> Request<'a> {
    /// Creates a new request from a client, method, and URL.
    ///
    /// # Arguments
    ///
    /// * `client` - The client making the request.
    /// * `method` - The HTTP method of the request.
    /// * `url` - The URL of the request.
    ///
    /// # Returns
    ///
    /// A new `Request` instance builder.
    pub(super) fn from_client(client: &'a WebClient, method: &str, url: &str) -> Self {
        Self {
            client,
            headers: HashMap::new(),
            method: method.to_string(),
            url: url.to_string(),
        }
    }

    /// Sets the header of the request to the given value
    ///
    /// # Arguments
    ///
    /// * `header` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// # Returns
    ///
    /// The modified `Request` instance.
    pub fn header(mut self, header: &str, value: &str) -> Self {
        self.headers.insert(header.to_string(), value.to_string());

        self
    }

    /// Gets the headers of this function ready to be sent to the server.
    /// Formatted for the HTTP Protocol
    ///
    /// # Arguments
    ///
    /// * `host` - The host to send this HTTP request to
    ///
    /// # Returns
    ///
    /// The headers of the request as a formatted string.
    fn get_headers(&self, host: &str) -> String {
        let mut lines = Vec::<String>::new();
        lines.push(format!("Host: {}", host));
        for (key, value) in self.headers.iter() {
            lines.push(format!("{}: {}", key, value))
        }

        return lines.join("\r\n");
    }

    /// Gets the beginning of the HTTP Payload
    ///
    /// # Arguments
    ///
    /// * `url` - The parsed URL that this request is being sent to
    /// * `path` - The path of the request.
    ///
    /// # Returns
    ///
    /// The full headers of the request as a formatted string.
    fn get_http_payload(&self, url: &Url, path: &str) -> Result<String> {
        // Using HTTP 1.1 Protocol
        let start = format!("{} {} HTTP/1.1\r\n", self.method, path);
        
        let server_name_raw = url.host_str().ok_or(anyhow!("Url has to have a host."))?;
        let headers = self.get_headers(server_name_raw);

        return Ok(format!("{}{}\r\n\r\n", start, headers));
    }


    /// Sends this request and returns the response of the server
    ///
    /// # Returns
    ///
    /// The response to the request.
    ///
    /// # Errors
    ///
    /// Returns an error if there was a problem sending the request or receiving the response.
    pub async fn send(self) -> Result<Response> {
        let url = Url::parse(&self.url)?;
        let mut path = url.path();
        if path.is_empty() {
            path = "/";
        }

        let proxy_conn = self.client.proxy().connect(&url).await?;
        let mut stream = self.client.create_connection(proxy_conn, &url).await?;

        let prepend = self.get_http_payload(&url, path)?;
        stream.write_all(prepend.as_bytes()).await?;

        let resp = Response::from_stream(stream).await?;
        return Ok(resp);
    }
}

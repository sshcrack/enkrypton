mod client;
mod response;
mod request;

/// This web client can send https requests across the tor network and parse/return the response.
pub use client::WebClient;
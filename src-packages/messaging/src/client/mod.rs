use lazy_static::lazy_static;

mod client;
mod tls;
/// This module contains some useful functions for the client side of the manager.
pub(crate) mod manager_ext;
mod proxy;
pub mod util;


pub use proxy::SocksProxy;
pub use client::MessagingClient;
pub use tls::*;

lazy_static! {
    pub static ref TOR_CLIENT: WebClient = WebClient::from_config().unwrap();
}
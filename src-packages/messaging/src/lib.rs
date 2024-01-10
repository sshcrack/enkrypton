/// Includes the client and read threads for handling packages
pub mod client;
/// General structs used to communicate between client and server and handle
/// incoming messages.
pub mod general;
/// General structs and traits for the websocket server hosted by the enkrypton binary
pub mod server;
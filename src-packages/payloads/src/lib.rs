/// Every packets that can be sent s2c or c2s
pub mod packets;
/// Payloads that are sent between frontend and backend
pub mod payloads;
/// Simple trait extension of the app handle to send payloads most easily
pub mod event;
/// Data structures that are used in payloads, can be exported to typescript using `cargo test`
pub mod data;
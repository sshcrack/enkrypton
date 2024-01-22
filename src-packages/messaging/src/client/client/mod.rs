/// Contains the core part of this library, the messaging client
mod index;
/// Used to detect if we should flush (so send) our message across the internet
mod flush;
/// This module contains the code to send a heartbeat every x seconds, so connection won't get interrupted.
pub(super) mod heartbeat;

pub use index::*;
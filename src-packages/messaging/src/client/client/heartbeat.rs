use std::{time::{Instant, Duration}, thread, sync::Arc};

use futures_util::SinkExt;
use log::{warn, debug};
use smol::block_on;
use tokio_tungstenite::tungstenite::Message;

use crate::general::HEARTBEAT;

use super::MessagingClient;

/// Used to send a ping to the server every HEARTBEAT ms.
pub(super) trait HeartbeatClient {
    /// Spawns the actual heartbeat thread
    fn spawn_heartbeat_thread(&mut self);
}

impl HeartbeatClient for MessagingClient {
    fn spawn_heartbeat_thread(&mut self) {
        // We don't want to spawn multiple heartbeat threads
        if self.heartbeat_thread.is_some() {
            warn!(
                "[CLIENT] Could not spawn heartbeat thread, already exists ({:?})",
                self
            );
            return;
        }

        // The mutex which will be used to send the ping
        let sender = self.write.clone();
        let handle = thread::Builder::new().name(format!("heartbeat-{}", self.receiver)).spawn(move || loop {
            // Measuring the time it took to send the ping, marking the beginning here
            let before = Instant::now();

            let mut write = block_on(sender.lock());

            // Feeding the ping to the queue of the websocket
            let temp = write.feed(Message::Ping(vec![]));
            let res = block_on(temp);
            drop(write);

            if let Err(e) = res {
                let err_msg = format!("{:?}", e);
                if err_msg.contains("AlreadyClosed") {
                    debug!("[CLIENT] Closing heartbeat thread...");
                    break;
                }

                warn!("[CLIENT] Could not send heartbeat: {:?}", e);
            }

            // The total time it took to send the ping
            let duration = before.elapsed();

            // Waiting for the remaining time
            let diff = HEARTBEAT.checked_sub(duration);
            let diff = diff.unwrap_or(Duration::new(0, 0));

            // Sleeping the remaining time
            thread::sleep(diff)
        }).unwrap();

        // Setting the heartbeat thread the current thread
        self.heartbeat_thread = Arc::new(Some(handle));
    }
}
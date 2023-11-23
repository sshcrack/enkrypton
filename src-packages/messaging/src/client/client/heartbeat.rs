use std::{time::{Instant, Duration}, thread, sync::Arc};

use futures_util::SinkExt;
use log::{warn, debug};
use smol::block_on;
use tokio_tungstenite::tungstenite::Message;

use crate::general::HEARTBEAT;

use super::MessagingClient;

pub(super) trait HeartbeatClient {
    fn spawn_heartbeat_thread(&mut self);
}

impl HeartbeatClient for MessagingClient {
    fn spawn_heartbeat_thread(&mut self) {
        if self.heartbeat_thread.is_some() {
            warn!(
                "[CLIENT] Could not spawn heartbeat thread, already exists ({:?})",
                self
            );
            return;
        }

        let sender = self.write.clone();
        let handle = thread::spawn(move || loop {
            let before = Instant::now();

            let mut write = block_on(sender.lock());

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

            let duration = before.elapsed();

            let diff = HEARTBEAT.checked_sub(duration);
            let diff = diff.unwrap_or(Duration::new(0, 0));

            thread::sleep(diff)
        });

        self.heartbeat_thread = Arc::new(Some(handle));
    }
}
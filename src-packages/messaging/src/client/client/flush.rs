use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, sleep, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::Result;
use futures_util::SinkExt;
use lazy_static::lazy_static;
use log::error;
use smol::block_on;
use tokio::sync::{Mutex, RwLock};

use super::WriteStream;
lazy_static! {
    // The delay to check if we should flush (so send all the messages in the queue)
    pub static ref FLUSH_DELAY: Duration = Duration::from_millis(100);
}

/// This struct is used to check if we should flush the websocket (so send all messages in the queue)
#[derive(Debug)]
pub(super) struct FlushChecker {
    /// The last time we flushed the websocket
    pub(super) last_update: Arc<RwLock<Instant>>,
    /// whether the current thread should exit
    pub(super) should_exit: Arc<AtomicBool>,
    #[allow(dead_code)]
    /// Not used for now
    write: Arc<Mutex<WriteStream>>,
    #[allow(dead_code)]
    /// Same here
    handle: JoinHandle<()>,
}

impl FlushChecker {
    /// Spawns the new thread which will check if the websocket should be flushed
    /// and returns its handle
    pub async fn spawn_handle(
        receiver: &str,
        last_update: Arc<RwLock<Instant>>,
        should_exit: Arc<AtomicBool>,
        write: Arc<Mutex<WriteStream>>,
    ) -> JoinHandle<()> {
        thread::Builder::new().name(format!("flush-checker-{}", receiver)).spawn(move || {
            while !should_exit.load(Ordering::Relaxed) {
                // Checking if we should flush and adding 5ms to make sure we don't miss the flush
                let to_wait = FLUSH_DELAY
                    .checked_sub(block_on(last_update.read()).elapsed())
                    .unwrap_or(Duration::from_secs(0))
                    + Duration::from_millis(5);
                sleep(to_wait);

                // Again checking if we should flush, if None we should flush now
                let to_wait = FLUSH_DELAY.checked_sub(block_on(last_update.read()).elapsed());
                if to_wait.is_some() {
                    continue;
                }

                // Actually flushing the websocket
                let res = block_on(block_on(write.lock()).flush());

                // Setting the last time the websocket was flushed to now
                *block_on(last_update.write()) = Instant::now();

                if let Err(e) = res {
                    error!("[CLIENT] Could not flush: {:?}", e);
                }
            }
        }).unwrap()
    }

    /// Constructs this struct and spawns the thread of this struct
    pub async fn new(receiver: &str, write: Arc<Mutex<WriteStream>>) -> Result<Self> {
        let last_update = Instant::now() - *FLUSH_DELAY - Duration::from_secs(1);
        let last_update = Arc::new(RwLock::new(last_update));
        let should_exit = Arc::new(AtomicBool::new(false));

        // Spawns the handle of this struct
        let handle =
            Self::spawn_handle(receiver, last_update.clone(), should_exit.clone(), write.clone()).await;
        let s = Self {
            // Just to make sure we flush on the first message
            last_update,
            should_exit,
            handle,
            write,
        };

        Ok(s)
    }

    /// Marks the queue as dirty, so we are waiting if there are any other messages coming in
    /// for FLUSH_DELAY time
    pub async fn mark_dirty(&self) {
        *self.last_update.write().await = Instant::now();
    }
}

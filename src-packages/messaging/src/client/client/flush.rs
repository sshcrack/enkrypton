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
    pub static ref FLUSH_DELAY: Duration = Duration::from_millis(100);
}

#[derive(Debug)]
pub(super) struct FlushChecker {
    pub(super) last_update: Arc<RwLock<Instant>>,
    pub(super) should_exit: Arc<AtomicBool>,
    #[allow(dead_code)]
    write: Arc<Mutex<WriteStream>>,
    #[allow(dead_code)]
    handle: JoinHandle<()>,
}

impl FlushChecker {
    pub async fn spawn_handle(
        last_update: Arc<RwLock<Instant>>,
        should_exit: Arc<AtomicBool>,
        write: Arc<Mutex<WriteStream>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
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

                *block_on(last_update.write()) = Instant::now();

                let res = block_on(block_on(write.lock()).flush());
                if let Err(e) = res {
                    error!("[CLIENT] Could not flush: {:?}", e);
                }
            }
        })
    }

    pub async fn new(write: Arc<Mutex<WriteStream>>) -> Result<Self> {
        let last_update = Instant::now() - *FLUSH_DELAY - Duration::from_secs(1);
        let last_update = Arc::new(RwLock::new(last_update));
        let should_exit = Arc::new(AtomicBool::new(false));

        let handle =
            Self::spawn_handle(last_update.clone(), should_exit.clone(), write.clone()).await;
        let s = Self {
            // Just to make sure we flush on the first message
            last_update,
            should_exit,
            handle,
            write,
        };

        Ok(s)
    }

    pub async fn mark_dirty(&self) {
        *self.last_update.write().await = Instant::now();
    }
}

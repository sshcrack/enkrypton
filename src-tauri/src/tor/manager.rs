use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

use crate::tor::consts::TOR_THREAD;

use super::{
    consts::{TOR_BINARY_PATH, TO_TOR_TX},
    integrity_check::check_integrity,
};

#[derive(Clone, serde::Serialize)]
pub struct StartTorPayload {
    pub message: String,
    pub progress: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum TorMessage {
    BootstrapProgress(f32),
    // To exit close the tx / rx channel.
    //Exit
}

pub async fn start_tor(on_event: impl Fn(StartTorPayload) -> ()) -> Result<()> {
    let already_started = TOR_THREAD.read().await;
    if already_started.is_some() {
        return Err(anyhow!("An Tor instance has already been started."));
    }

    on_event(StartTorPayload {
        message: "Checking integrity...".to_owned(),
        progress: 0.0,
    });
    check_integrity()?;

    on_event(StartTorPayload {
        message: "Starting tor...".to_owned(),
        progress: 0.5,
    });

    info!("Starting handle...");
    let handle = thread::spawn(move || {
        let res = block_on(tor_main_loop());
        if res.is_ok() {
            info!("TOR thread has finished!");
            return;
        }

        let err = res.unwrap_err();
        error!("TOR thread has failed: {:#?}", err);
    });

    info!("Storing handle in RW lock...");
    let mut e = TOR_THREAD.write().await;
    *e = Some(handle);

    Ok(())
}

pub async fn stop_tor() {
    let tx = TO_TOR_TX.read().await;
    tx.as_ref().unwrap().close();
}

/**
 * Spawns the tor process
 * Controls and interprets the output of the tor process
 */
async fn tor_main_loop() -> Result<()> {
    let (tx, rx) = async_channel::unbounded::<TorMessage>();
    TO_TOR_TX.write().await.replace(tx);

    debug!("Starting communicator");
    let mut child = Command::new(TOR_BINARY_PATH.clone())
        .stdout(Stdio::piped())
        .spawn()?;

    let should_exit = Arc::new(AtomicBool::new(false));
    debug!("Running main loop");

    const BOOTSTRAP_MSG: &str = "[notice] Bootstrapped ";

    let temp = should_exit.clone();
    let stdout = child.stdout.take().unwrap();
    let handle = thread::spawn(move || {
        let mut stdout = BufReader::new(stdout);
        while !temp.load(Ordering::Relaxed) {
            let mut buf = String::new();
            match stdout.read_line(&mut buf) {
                Ok(_) => {
                    let msg = buf.to_string();
                    if msg.contains(BOOTSTRAP_MSG) {
                        let split: Vec<&str> = msg.split(BOOTSTRAP_MSG)
                            .collect();

                        let split = split.get(0).unwrap_or(&"");
                        let split: Vec<&str> = split.split(" ").collect();

                        let percentage = split.get(0).unwrap_or(&"0").replace("%", "");

                        let info = split.get(1).unwrap_or(&":no info");
                        let info: Vec<&str> = info.split(':').collect();

                        let info = info.get(1).unwrap_or(&"no info");


                        info!("TOR Percentage: {} -- {}", percentage, info)
                    }

                    info!("TOR: {}", buf.to_string());
                }
                Err(e) => error!("an error!: {:?}", e),
            }
        }
    });

    while !rx.is_closed() && !should_exit.load(Ordering::Relaxed) {
        if rx.len() != 0 {
            debug!("Waiting for message");
            let msg = rx.recv().await?;
            debug!("TOR: Received a new message: {:#?}", msg)
        }
    }

    debug!("Exiting...");
    should_exit.store(true, Ordering::Relaxed);
    if !rx.is_closed() {
        rx.close();
    }

    child.kill()?;

    debug!("Waiting for handle to exit...");
    handle.join().expect("Error waiting for tor handle to exit");
    Ok(())
}

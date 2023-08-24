use std::{
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, yield_now},
};

use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

use crate::tor::{
    consts::{FROM_TOR_RX, TOR_THREAD},
    parser::stdout::handle_tor_stdout,
};

use super::{
    consts::{TOR_BINARY_PATH, TO_TOR_TX},
    integrity_check::check_integrity,
};

#[derive(Clone, serde::Serialize)]
pub struct StartTorPayload {
    pub message: String,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub enum Tor2ClientMsg {
    BootstrapProgress(f32, String),
    StatusMsg(String), // To exit close the tx / rx channel.
                       //Exit
}

#[derive(Debug, Clone)]
pub enum Client2TorMsg {}

pub async fn start_tor(on_event: impl Fn(StartTorPayload) -> ()) -> Result<()> {
    let already_started = TOR_THREAD.read().await;
    if already_started.is_some() {
        return Err(anyhow!("An Tor instance has already been started."));
    }

    drop(already_started);

    info!("Checking integrity...");
    on_event(StartTorPayload {
        message: "Checking integrity...".to_owned(),
        progress: 0.0,
    });
    check_integrity()?;

    on_event(StartTorPayload {
        message: "Starting tor...".to_owned(),
        progress: 0.3,
    });

    debug!("Creating unbounded channels...");
    let (to_tx, to_rx) = async_channel::unbounded::<Client2TorMsg>();
    let (from_tx, from_rx) = async_channel::unbounded::<Tor2ClientMsg>();

    debug!("Writing to rwlock...");
    TO_TOR_TX.write().await.replace(to_tx);
    FROM_TOR_RX.write().await.replace(from_rx.clone());

    let handle = thread::spawn(move || {
        let res = block_on(tor_main_loop(from_tx, to_rx));
        if res.is_ok() {
            info!("TOR: thread has finished!");
            return;
        }

        let err = res.unwrap_err();
        error!("TOR thread has failed: {:#?}", err);
    });

    debug!("Writing tor thread");
    TOR_THREAD.write().await.replace(handle);

    debug!("Waiting for tor to start...");

    let mut is_done = true;
    while !is_done {
        debug!("Reading msg");
        let msg = from_rx.recv().await?;
        match msg {
            Tor2ClientMsg::BootstrapProgress(prog, status) => {
                debug!("Received new msg: {} {}", prog, status);
                if prog == 1.0 {
                    is_done = true;
                }

                on_event(StartTorPayload {
                    message: status,
                    progress: prog / 3.0 + 0.6,
                });
            }
            _ => {}
        }
    }

    debug!("Done");
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
async fn tor_main_loop(
    from_tx: Sender<Tor2ClientMsg>,
    to_rx: Receiver<Client2TorMsg>,
) -> Result<()> {
    debug!("Starting tor...");
    let mut child = Command::new(TOR_BINARY_PATH.clone())
        .stdout(Stdio::piped())
        .spawn()?;

    let should_exit = Arc::new(AtomicBool::new(false));

    let temp = should_exit.clone();
    let stdout = child.stdout.take().unwrap();
    let handle = thread::spawn(move || handle_tor_stdout(temp, stdout, from_tx));

    yield_now();
    while !to_rx.is_closed() && !should_exit.load(Ordering::Relaxed) {
        if to_rx.len() != 0 {
            debug!("TOR: Reading message...");
            let msg = to_rx.recv().await?;
            debug!("TOR: Received a new message: {:#?}", msg)
        }
    }

    debug!("Exiting...");
    should_exit.store(true, Ordering::Relaxed);
    if !to_rx.is_closed() {
        to_rx.close();
    }

    child.kill()?;

    handle
        .join()
        .expect("Error waiting for tor handle to exit")
        .await
        .unwrap();

    debug!("Exit done");
    Ok(())
}

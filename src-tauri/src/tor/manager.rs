use std::{process::Command, thread, fmt::Display, fs::File};

use anyhow::{Result, anyhow};
use log::{info, debug, error};
use subprocess::{Exec, Redirection};
use tauri::async_runtime::block_on;

use crate::tor::consts::TOR_THREAD;

use super::{integrity_check::check_integrity, consts::{TO_TOR_TX, TOR_BINARY_PATH}};

#[derive(Clone, serde::Serialize)]
pub struct StartTorPayload {
    pub message: String,
    pub progress: f32
}



#[derive(Debug, Clone, Copy)]
pub enum TorMessage {
    StartProgress(f32),
    // To exit close the tx / rx channel.
    //Exit
}


pub async fn start_tor(on_event: impl Fn(StartTorPayload) -> ()) -> Result<()> {
    let already_started = TOR_THREAD.read().await;
    if already_started.is_some() {
        return Err(anyhow!("An Tor instance has already been started."))
    }

    on_event(StartTorPayload {
        message: "Checking integrity...".to_owned(),
        progress: 0.0
    });
    check_integrity()?;

    on_event(StartTorPayload {
        message: "Starting tor...".to_owned(),
        progress: 0.5
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

/**
 * Spawns the tor process
 * Controls and interprets the output of the tor process
 */
async fn tor_main_loop() -> Result<()>{
    let (tx, rx) = async_channel::unbounded::<TorMessage>();
    TO_TOR_TX.write().await.replace(tx);

    let mut communicator = Exec::cmd(TOR_BINARY_PATH.clone())
        .arg("-f")
        .arg("torrc")
        .communicate()?;

    let mut should_exit = false;
    while !rx.is_closed() && !should_exit {
        if rx.len() != 0 {
            let msg = rx.recv().await?;
            debug!("TOR: Received a new message: {:#?}", msg)
        }

        let (out, parsed_err) = communicator.read_string()?;
        if out.is_none() {
            continue;
        }

        if parsed_err.is_some() {
            debug!("Parsing errors for tor: {}", parsed_err.unwrap())
        }

        let out = out.unwrap();
        if !out.is_empty() {
            info!("TOR: {}", out);
        }
    }

    if !rx.is_closed() {
        rx.close();
    }

    Ok(())
}




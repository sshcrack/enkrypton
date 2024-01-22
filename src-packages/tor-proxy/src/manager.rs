use std::{
    fs::File,
    io::Write,
    thread::{self},
};

use anyhow::{anyhow, bail, Result};
use payloads::payloads::StartTorPayload;
use shared::{get_torrc, config::CONFIG};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

use crate::{misc::{integrity_check::check_integrity, tools::{get_to_tor_tx, get_from_tor_rx}, messages::{Client2TorMsg, Tor2ClientMsg, TorStartError}}, consts::{TOR_START_LOCK, TOR_THREAD}, mainloop::tor_main_loop, service::get_service_hostname, config::ConfigExt};

/// Starts tor and accepts a function that will be used to report about the progress
///
/// # Arguments
///
/// * `on_event` - The function that will be used to report about the progress
pub async fn start_tor(on_event: impl Fn(StartTorPayload) -> ()) -> Result<()> {
    let already_started = TOR_THREAD.read().await;
    if already_started.is_some() {
        return Err(anyhow!("An Tor instance has already been started."));
    }

    let mut lock = TOR_START_LOCK.write().await;
    drop(already_started);

    info!("Checking integrity...");
    on_event(StartTorPayload {
        message: "Checking integrity / writing torrc...".to_owned(),
        progress: 0.0,
    });
    check_integrity()?;

    write_torrc().await?;

    on_event(StartTorPayload {
        message: "Starting tor...".to_owned(),
        progress: 0.3,
    });

    debug!("Creating unbounded channels...");
    debug!("Writing to rwlock...");

    // Starts the tor thread
    let handle = thread::Builder::new().name("tor-mainloop".to_string()).spawn(move || {
        let res = block_on(tor_main_loop());
        if res.is_ok() {
            info!("TOR: thread has finished!");
            return;
        }

        let err = res.unwrap_err();
        error!("TOR thread has failed: {:#?}", err);
    }).unwrap();

    debug!("Writing tor thread");
    TOR_THREAD.write().await.replace(handle);

    debug!("Waiting for tor to start...");
    // Handle tor startup messages
    let rx = get_from_tor_rx().await;
    loop {
        if rx.len() > 0 {
            let msg = rx.recv().await?;
            match msg {
                Tor2ClientMsg::BootstrapProgress(progress, status) => {
                    on_event(StartTorPayload {
                        progress: progress / 3.0 + 2.0 / 3.0,
                        message: status,
                    });

                    if progress == 1.0 {
                        // Tor is done starting up so we are exiting the read loop
                        break;
                    }
                }
                Tor2ClientMsg::ExitMsg(status, logs) => {
                    bail!(TorStartError { logs, status });
                }
                _ => {}
            }
        }
    }

    // And gets the hostname to log it
    let hostname = get_service_hostname(true).await?;
    info!("Onion Service Hostname is {:?}", hostname);
    *lock = true;
    Ok(())
}

/// Writes the configuration file for tor
pub async fn write_torrc() -> Result<()> {
    let buf = get_torrc();
    let mut file = File::create(buf)?;

    let config = CONFIG.to_text().await?;
    file.write_all(config.as_bytes())?;

    Ok(())
}

/// Waits for tor to exit and blocks the main handle
pub async fn wait_for_exit() {
    let mut handle = TOR_THREAD.write().await;

    info!("Waiting for handle");
    handle
        .take()
        .expect("Could not wait for tor client to exit")
        .join()
        .expect("msg");
}

/// Sends the exit signal to tor
pub async fn stop_tor() -> Result<()> {
    let handle = TOR_THREAD.read().await;
    if !handle.is_some() {
        return Err(anyhow!("Could not stop tor, tor is not running"));
    }
    drop(handle);

    let state = get_to_tor_tx().await;
    info!("Sending exit signal...");
    state.send(Client2TorMsg::Exit()).await?;

    Ok(())
}
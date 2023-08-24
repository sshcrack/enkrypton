use std::{thread::{self}, process::ExitStatus};

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

use crate::tor::{misc::{integrity_check::check_integrity, tools::{get_from_tor_rx, get_to_tor_tx}, payloads::{Tor2ClientMsg, StartTorError}}, mainloop::tor_main_loop};

use super::{consts::TOR_THREAD, misc::payloads::StartTorPayload};

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
    debug!("Writing to rwlock...");

    let handle = thread::spawn(move || {
        let res = block_on(tor_main_loop());
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
    let rx = get_from_tor_rx().await;
    loop {
        if rx.len() > 0 {
            let msg = rx.recv().await?;
            match msg {
                Tor2ClientMsg::BootstrapProgress(prog, status) => {
                    on_event(StartTorPayload {
                        progress: prog / 3.0 + 0.6,
                        message: status
                    });

                    if prog == 1.0 {
                        break;
                    }
                }
                Tor2ClientMsg::ExitMsg(status, logs) => {
                    return Err(StartTorError(status, logs));
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub async fn wait_for_exit() {
    let mut handle = TOR_THREAD.write().await;

    info!("Waiting for handle");
    handle
        .take()
        .expect("Could not wait for tor client to exit")
        .join()
        .expect("msg");
}

pub async fn stop_tor() -> anyhow::Result<()> {
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

pub async fn wait_and_stop_tor() -> anyhow::Result<()> {
    stop_tor().await?;
    wait_for_exit().await;
    Ok(())
}

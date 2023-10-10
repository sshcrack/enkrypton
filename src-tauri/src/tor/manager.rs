use std::{
    fs::File,
    io::Write,
    thread::{self},
};

use anyhow::{anyhow, bail, Result};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

use crate::{
    payloads::StartTorPayload,
    tor::{
        mainloop::tor_main_loop,
        misc::{
            integrity_check::check_integrity,
            messages::{Client2TorMsg, Tor2ClientMsg, TorStartError},
            tools::{get_from_tor_rx, get_to_tor_tx},
        }, service::get_service_hostname,
    },
};

use super::{
    config::CONFIG,
    consts::{get_torrc, TOR_THREAD, TOR_START_LOCK},
};

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
                        progress: prog / 3.0 + 2.0 / 3.0,
                        message: status,
                    });

                    if prog == 1.0 {
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

    let hostname = get_service_hostname()?;
    info!("Onion Service Hostname is {:?}", hostname);
    *lock = true;
    Ok(())
}

pub async fn write_torrc() -> Result<()> {
    let buf = get_torrc();
    let mut file = File::create(buf)?;

    file.write_all(CONFIG.to_text().as_bytes())?;

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

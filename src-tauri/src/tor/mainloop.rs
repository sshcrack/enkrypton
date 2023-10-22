use std::{
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self},
};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

use crate::tor::{
    consts::{get_torrc, TOR_ZIP_PATH},
    misc::{messages::Client2TorMsg, tools::get_to_tor_rx},
    parser::stdout::handle_tor_stdout,
};
use anyhow::{anyhow, Result};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/**
 * Spawns the tor process
 * Controls and interprets the output of the tor process
 */
pub(super) async fn tor_main_loop() -> Result<()> {
    info!("Starting tor...");
    let child = Command::new(TOR_ZIP_PATH.clone())
        .args(["-f", &get_torrc().to_string_lossy()])
        .stdout(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    let id = child.id();

    let should_exit = Arc::new(AtomicBool::new(false));

    let temp = should_exit.clone();

    let handle = thread::spawn(move || {
        let res = block_on(handle_tor_stdout(temp, child));
        if res.is_ok() {
            info!("TOR: Thread finished");
        } else {
            let err = res.unwrap_err();
            error!("TOR: failed {}", err);
        }
    });

    let rx = get_to_tor_rx().await;
    while !rx.is_closed() && !should_exit.load(Ordering::Relaxed) {
        if rx.len() > 0 {
            let msg = rx.recv().await?;
            match msg {
                Client2TorMsg::Exit() => {
                    debug!("Got exit signal");
                    break;
                }
            }
        }
    }

    should_exit.store(true, Ordering::Relaxed);

    let s = System::new_all();
    if let Some(process) = s.process(Pid::from_u32(id)) {
        process.kill();
    }

    info!("Waiting for handle to exit...");
    handle
        .join()
        .or(Err(anyhow!("Could not wait for tor handle to exit")))?;

    info!("Exited.");
    Ok(())
}

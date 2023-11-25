#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self},
};

use shared::get_torrc;
#[cfg(target_family="unix")]
use shared::get_root_dir;
#[cfg(target_family="unix")]
use std::env;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use tauri::async_runtime::block_on;

use crate::{
    consts::TOR_BINARY_PATH,
    misc::{messages::Client2TorMsg, tools::get_to_tor_rx},
    parser::stdout::handle_tor_stdout,
};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Spawns the tor process
/// Controls and interprets the output of the tor process
pub(super) async fn tor_main_loop() -> Result<()> {
    info!("Starting tor...");

    // Starts tor
    let mut child = Command::new(TOR_BINARY_PATH.clone());
    child.args(["-f", &get_torrc().to_string_lossy()]);
    child.stdout(Stdio::piped());

    #[cfg(target_family = "unix")]
    // We need to tell Linux about the additional dynamic libraries provided by tor
    {
        let ld = env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let ld = format!("{}:{}", ld, get_root_dir().to_string_lossy());
        child.env("LD_LIBRARY_PATH", ld);
    }

    #[cfg(target_os = "windows")]
    // And we don't want to create a new window for the tor process
    child.creation_flags(CREATE_NO_WINDOW);

    // Actually spawning tor
    let child = child.spawn()?;
    let id = child.id();

    // Wether the tor handler thread should exit
    let should_exit = Arc::new(AtomicBool::new(false));

    let temp = should_exit.clone();


    // Spawns the tor thread to handle tor stdout
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
    // If we should exit, break and tell the tor process to exit as well
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

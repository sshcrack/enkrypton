use std::{
    io::{BufRead, BufReader},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Result, anyhow};
use async_channel::Sender;
use log::{debug, warn, error, info};

use crate::tor::{manager::stop_tor, misc::{tools::get_from_tor_tx, payloads::Tor2ClientMsg}, consts::MAX_LOG_SIZE};

use super::messages::{BOOTSTRAP_MSG, WARN_MSG, NOTICE_MSG, ERR_MSG};

pub async fn handle_tor_stdout(should_exit: Arc<AtomicBool>, mut child: Child) -> Result<()> {
    let stdout = child.stdout.take().unwrap();
    let tx = get_from_tor_tx().await;


    let mut stdout = BufReader::new(stdout);
    let mut logs = Vec::<String>::with_capacity(10);
    while !should_exit.load(Ordering::Relaxed) {
        let res = child.try_wait()?;

        if res.is_some() {
            let intentional = should_exit.load(Ordering::Relaxed);
            debug!("Process exited intentional: {}", intentional);
            if intentional {
                debug!("Intentional exit. Exiting...");
                break;
            }

            let err_stat = res.unwrap();

            stop_tor().await?;
            return Err(anyhow!("Tor exited with code {}", err_stat));
        }

        let mut buf = String::new();
        match stdout.read_line(&mut buf) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    continue;
                }

                let msg = buf.trim_end_matches("\r\n").trim_end_matches('\n');
                let msg = msg.to_string();
                let res = check_msg(&msg, &tx).await;
                if res.is_err() {
                    error!("Could not process msg {}:\n{}", &msg, res.unwrap_err());
                }

                if logs.len() > *MAX_LOG_SIZE {
                    logs.remove(0);
                }

                logs.push(msg);
            }
            Err(e) => error!("an error!: {:?}", e),
        }
    }

    debug!("Handle done");
    Ok(())
}

async fn check_msg(msg: &str, tx: &Sender<Tor2ClientMsg>) -> Result<()> {
    let msg = msg.to_string();
    if msg.contains(BOOTSTRAP_MSG) {
        handle_bootstrap(&msg, tx).await?;
        return Ok(());
    }

    //TODO remove duplicate if functions here
    if msg.contains(WARN_MSG) {
        warn!("TOR: {}", msg);
        tx.send(Tor2ClientMsg::WarnMsg(msg)).await?;
        return Ok(());
    }

    if msg.contains(ERR_MSG) {
        error!("TOR: {}", msg);
        tx.send(Tor2ClientMsg::ErrorMsg(msg)).await?;
        return Ok(());
    }

    if msg.contains(NOTICE_MSG) {
        debug!("TOR: {}", msg);
        tx.send(Tor2ClientMsg::NoticeMsg(msg)).await?;
        return Ok(());
    }


    Ok(())
}

async fn handle_bootstrap(msg: &str, tx: &Sender<Tor2ClientMsg>) -> Result<()> {
    let split: Vec<&str> = msg.split(BOOTSTRAP_MSG).collect();

    let main_fragment = split.get(1).unwrap_or(&"");
    let mut space_split: Vec<&str> = main_fragment.split(" ").collect();

    // removing the first element so array can be joined together later for status message
    //TODO this panics, replace it by other things and make code cleaner pls thx
    let percentage = space_split
        .remove(0)
        // remove percentage symbol
        .replace("%", "");

    // going with a default of -1 percent if reading fails
    let percentage = percentage.parse::<f32>().unwrap_or(-100.0) / 100.0;

    let info: Vec<&str> = main_fragment.split(": ").collect();
    let info = info.get(1).unwrap_or(&"no info");

    tx.send(Tor2ClientMsg::BootstrapProgress(
        percentage,
        info.to_string(),
    ))
    .await?;

    Ok(())
}

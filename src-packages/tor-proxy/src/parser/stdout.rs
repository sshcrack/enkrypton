use std::{
    io::{BufRead, BufReader},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Result};
use async_channel::Sender;
use log::{debug, error, warn};

use crate::{
    consts::MAX_LOG_SIZE,
    manager::stop_tor,
    misc::{messages::Tor2ClientMsg, tools::get_from_tor_tx},
};

use super::messages::{BOOTSTRAP_MSG, ERR_MSG, NOTICE_MSG, WARN_MSG};

pub async fn handle_tor_stdout(should_exit: Arc<AtomicBool>, mut child: Child) -> Result<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or(anyhow!("Could not take child stdout"))?;

    let tx = get_from_tor_tx().await;

    let mut stdout = BufReader::new(stdout);
    let mut logs = Vec::<String>::with_capacity(10);
    while !should_exit.load(Ordering::Relaxed) {
        let res = child.try_wait()?;

        if let Some(err_stat) = res {
            let intentional = should_exit.load(Ordering::Relaxed);
            if intentional {
                debug!("Intentional exit. Exiting...");
                break;
            }

            error!("Tor exited with code {:?} logs are: \n---\n{}\n---\nStopping...", err_stat, logs.join("\n"));
            tx.send(Tor2ClientMsg::ExitMsg(err_stat, logs)).await?;
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

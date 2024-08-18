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

/// Handles the stdout of the tor process
///
/// # Arguments
///
/// * `should_exit` - Whether the tor proxy and thread should be exited
/// * `child` - The spawned tor proxy process to read from
///
pub async fn handle_tor_stdout(should_exit: Arc<AtomicBool>, mut child: Child) -> Result<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or(anyhow!("Could not take child stdout"))?;

    // First of all we need the channel to send updates
    let tx = get_from_tor_tx().await;

    // Then we are creating a new reader from the tor process just spawned
    let mut stdout = BufReader::new(stdout);
    // Keeping the 10 last logs in memory
    let mut logs = Vec::<String>::with_capacity(10);

    // And exiting only if a exit signal was sent
    while !should_exit.load(Ordering::Relaxed) {
        // checking if the tor process has exited
        let res = child.try_wait()?;

        if let Some(err_stat) = res {
            // If it was intentional, we don't report anything
            let intentional = should_exit.load(Ordering::Relaxed);
            if intentional {
                debug!("Intentional exit. Exiting...");
                break;
            }

            // If not, report to frontend and send over the channel
            error!("Tor exited with code {:?} logs are: \n---\n{}\n---\nStopping...", err_stat, logs.join("\n"));
            tx.send(Tor2ClientMsg::ExitMsg(err_stat, logs)).await?;
            stop_tor().await?;
            return Err(anyhow!("Tor exited with code {}", err_stat));
        }

        // We are reading the new stdout line to the buffer
        let mut buf = String::new();
        match stdout.read_line(&mut buf) {
            Ok(bytes_read) => {
                // We could successfully read some bytes, so process them
                if bytes_read == 0 {
                    // Obviously skip empty strings
                    continue;
                }

                // Trimming messages to remove trailing \r\n
                let msg = buf.trim_end_matches("\r\n").trim_end_matches('\n');
                let msg = msg.to_string();

                // Process the tor message
                let res = handle_msg(&msg, &tx).await;
                if res.is_err() {
                    error!("Could not process msg {}:\n{}", &msg, res.unwrap_err());
                }

                // Removes excess logs
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

/// Used to handle a single tor message and send updates to the bootstrap
async fn handle_msg(msg: &str, tx: &Sender<Tor2ClientMsg>) -> Result<()> {
    let msg = msg.to_string();
    if msg.contains(BOOTSTRAP_MSG) {
        // Handle bootstrap messages
        handle_bootstrap(&msg, tx).await?;
        return Ok(());
    }

    if msg.contains(WARN_MSG) {
        warn!("TOR: {}", msg);
        // Warn messages
        tx.send(Tor2ClientMsg::WarnMsg(msg)).await?;
        return Ok(());
    }

    if msg.contains(ERR_MSG) {
        // Again, error messages
        error!("TOR: {}", msg);
        tx.send(Tor2ClientMsg::ErrorMsg(msg)).await?;
        return Ok(());
    }

    if msg.contains(NOTICE_MSG) {
        // And finally notice messages
        debug!("TOR: {}", msg);
        tx.send(Tor2ClientMsg::NoticeMsg(msg)).await?;
        return Ok(());
    }

    Ok(())
}

async fn handle_bootstrap(msg: &str, tx: &Sender<Tor2ClientMsg>) -> Result<()> {
    let split: Vec<&str> = msg.split(BOOTSTRAP_MSG).collect();

    // Splitting the message into the main fragment and the percentage
    let main_fragment = split.get(1).unwrap_or(&"");
    let space_split: Vec<&str> = main_fragment.split(" ").collect();

    // removing the first element so array can be joined together later for status message
    let percentage = space_split
        .get(0)
        .ok_or(anyhow!("Could not get percentage from bootstrap message"))?
        // remove percentage symbol
        .replace("%", "");

    // going with a default of -1 percent if reading fails
    let percentage = percentage.parse::<f32>().unwrap_or(-100.0) / 100.0;

    let info: Vec<&str> = main_fragment.split(": ").collect();
    let info = info.get(1).unwrap_or(&"no info");

    // And sending the percentage update to the frontend
    tx.send(Tor2ClientMsg::BootstrapProgress(
        percentage,
        info.to_string(),
    ))
    .await?;

    Ok(())
}

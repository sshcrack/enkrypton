use std::{sync::{atomic::{Ordering, AtomicBool}, Arc}, io::{BufReader, BufRead}, process::ChildStdout};

use anyhow::Result;
use async_channel::Sender;
use log::{info, error, debug};

use crate::tor::manager::Tor2ClientMsg;

pub async fn handle_tor_stdout(should_exit: Arc<AtomicBool>, stdout: ChildStdout, tx: Sender<Tor2ClientMsg>) -> Result<()> {
    const BOOTSTRAP_MSG: &str = "[notice] Bootstrapped ";

    debug!("handle");
    let mut stdout = BufReader::new(stdout);
    while !should_exit.load(Ordering::Relaxed) {
        let mut buf = String::new();
        match stdout.read_line(&mut buf) {
            Ok(_) => {
                debug!("msg");
                let msg = buf.trim_end_matches("\r\n").trim_end_matches('\n');
                if msg.replace(" ", "").is_empty() {
                    continue;
                }

                info!("TOR: {}", buf.to_string());
                if msg.contains(BOOTSTRAP_MSG) {
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

                    debug!("Sending msg...");
                    tx.send(Tor2ClientMsg::BootstrapProgress(percentage, info.to_string())).await?;
                    continue;
                }

                tx.send(Tor2ClientMsg::StatusMsg(msg.to_string())).await?;
            }
            Err(e) => error!("an error!: {:?}", e),
        }
    }

    Ok(())
}

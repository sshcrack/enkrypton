use std::{fmt, process::ExitStatus};

#[derive(Debug, Clone)]
pub enum Tor2ClientMsg {
    BootstrapProgress(f32, String),
    WarnMsg(String),
    NoticeMsg(String),
    ErrorMsg(String),
    /* Sent when the tor client unexpectedly closed arguments are: exitStatus and last {MAX_LOG_SIZE} logs (default 20 logs being kept)*/
    ExitMsg(ExitStatus, Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Client2TorMsg {
    /* make sure just to send this one when we REALLY want the program to exit, bit difficult to start tor all over again
     */
    Exit(),
}

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct TorStartError {
    pub logs: Vec<String>,
    pub status: ExitStatus,
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for TorStartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.status.code();
        let code = code
            .and_then(|e| Some(e.to_string()))
            .unwrap_or("<invalid_code>".to_string());

        let mut last_logs = Vec::<String>::new();
        let size = self.logs.len();

        let min_size = if size <= 3 { 0 } else { size - 3 };
        for i in min_size..size {
            let el = self.logs.get(i);
            if let Some(e) = el {
                last_logs.push(e.to_string());
            }
        }

        if last_logs.is_empty() {
            last_logs.push("No logs".to_string());
        }

        write!(
            f,
            "Tor could not start correctly and exited with code {}  and following logs:\n{}",
            code,
            last_logs.join("\n")
        )
    }
}

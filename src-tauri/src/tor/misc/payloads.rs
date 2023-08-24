use std::{fmt, process::ExitStatus};

#[derive(Clone, serde::Serialize)]
pub struct StartTorPayload {
    pub progress: f32,
    pub message: String,
}

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
pub struct StartTorError {
    pub logs: Vec<String>,
    pub status: ExitStatus,
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for StartTorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.status.code();
        let code = if code.is_some() {
            code.unwrap().to_string()
        } else {
            "<invalid code>".to_owned()
        };

        let mut last_logs = Vec::<String>::new();
        let size = self.logs.len();

        for i in (size - 3)..size {
            let el = self.logs.get(i);
            if el.is_none() {
                continue;
            }

            last_logs.push(el.unwrap().to_string());
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

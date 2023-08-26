use log::{error, warn};
use tauri::{async_runtime::{self}, App, Manager};

use crate::{tor::{manager::{self}, misc::messages::TorStartError}, payloads::start_tor::TorStartupErrorPayload};

pub fn startup(app: &mut App) {
    let window = app.get_window("main").unwrap();

    #[cfg(debug_assertions)] // only include this code on debug builds
    {
        window.open_devtools();
    }

    let splashscreen_window = app.get_window("splashscreen").unwrap();
    let temp = splashscreen_window.clone();

    let _env = app.env();
    temp.once_global("splashscreen_ready", move |_event| {
        async_runtime::spawn(async move {
            let temp = splashscreen_window.clone();
            let res = manager::start_tor(move |start_payload| {
                let res = temp.app_handle().emit_all("tor_start", start_payload);
                if res.is_ok() {
                    return;
                }

                warn!("Tor start could not send payload {:?}", res.unwrap_err())
            })
            .await;

            if res.is_ok() {
                window.show().unwrap();
                splashscreen_window.close().unwrap();
            }

            if res.is_err() {
                let err: anyhow::Error = res.unwrap_err();
                window.close().unwrap();

                let mut payload = TorStartupErrorPayload {
                    message: err.to_string(),
                    error_code: None,
                    logs: None,
                };

                let start_err = err.downcast::<TorStartError>();
                if start_err.is_ok() {
                    let start_err = start_err.unwrap();

                    payload.error_code = start_err.status.code();
                    payload.logs = Some(start_err.logs);
                }

                error!("Could not start tor: {}", payload.message);
                splashscreen_window
                    .app_handle()
                    .emit_all("tor_start_error", payload)
                    .unwrap();
            }
        });
    });
}

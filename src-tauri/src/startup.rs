use log::{warn, error};
use tauri::{App, Manager, async_runtime};

use crate::tor::{manager, misc::payloads::TorStartError};

pub struct TorStartupErrorPayload {
    message: Option<String>,
    error_code: Option<i32>,
    logs: Option<Vec<String>>
}

pub fn startup(app: &mut App) {
    let window = app.get_window("main").unwrap();

            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                window.open_devtools();
            }

            let splashscreen_window = app.get_window("splashscreen").unwrap();
            let temp = splashscreen_window.clone();

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

                        let tor_err: Result<TorStartError, ()> = err.try_into();
                        error!("Could not start tor: {}", err);

                        splashscreen_window
                            .app_handle()
                            .emit_all("tor_start_error", err.to_string())
                            .unwrap();
                    }
                });
            });
}
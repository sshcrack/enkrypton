use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use log::{debug, error, warn};
use payloads::{payloads::{TorStartupErrorPayload, splashscreen::SplashscreenClosedPayload}, event::AppHandleExt};
use shared::APP_HANDLE;
use tauri::{
    async_runtime::{self, block_on},
    App, Manager,
};
use tor_proxy::{manager, misc::messages::TorStartError};

use crate::util::on_exit;

/// The whole startup process of this app
pub fn startup(app: &mut App) {
    let mut state = APP_HANDLE.blocking_write();
    *state = Some(app.handle());

    drop(state);

    // Window is the main window
    let window = app.get_window("main").unwrap();

    #[cfg(debug_assertions)] // only include this code on debug builds
    {
        window.open_devtools();
    }

    // The splashscreen to close later
    let splashscreen_window = app.get_window("splashscreen").unwrap();
    let temp = splashscreen_window.clone();

    let _env = app.env();
    temp.once_global("splashscreen_ready", move |_event| {
        // Starting tor if the splashscreen is ready
        async_runtime::spawn(async move {
            let temp = splashscreen_window.clone();
            let res = manager::start_tor(move |start_payload| {
                let res = temp.app_handle().emit_payload(start_payload);
                if res.is_ok() {
                    return;
                }

                warn!("Tor start could not send payload {:?}", res.unwrap_err())
            })
            .await;

            if res.is_ok() {
                // After starting tor, close the splashscreen and show the main window
                #[cfg(debug_assertions)]
                window.open_devtools();
                window.show().unwrap();
                splashscreen_window.close().unwrap();
                splashscreen_window.app_handle().emit_payload(SplashscreenClosedPayload { }).unwrap();
            }

            // If there is any error, report it
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

                // Tell the splashscreen about the errors
                error!("Could not start tor: {}", payload.message);
                splashscreen_window
                    .app_handle()
                    .emit_payload(payload)
                    .unwrap();
            }
        });
    });

    let term = Arc::new(AtomicBool::new(false));

    // Handle the SIGINT signal and stop tor first
    let handle = app.handle();
    thread::spawn(move || {
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();
        while !term.load(Ordering::Relaxed) {}

        debug!("Running stop on main thread");
        let r = block_on(on_exit());

        handle.exit(0);

        if let Err(e) = r {
            error!("Could not exit: {}", e);
        }
    });
}

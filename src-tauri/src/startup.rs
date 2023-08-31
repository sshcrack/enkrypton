use std::{
    f32::consts::E,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use log::{debug, error, warn};
use tauri::{
    async_runtime::{self, block_on},
    App, Manager,
};

use crate::{
    payloads::start_tor::TorStartupErrorPayload,
    tor::{
        manager::{self},
        misc::messages::TorStartError,
    },
};

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

    let term = Arc::new(AtomicBool::new(false));

    let handle = app.handle();
    thread::spawn(move || {
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();
        while !term.load(Ordering::Relaxed) {}

        debug!("Running stop on main thread");
        let r = handle.run_on_main_thread(|| {
            block_on(manager::wait_and_stop_tor()).unwrap();
        });

        handle.exit(0);

        if let Err(e) = r {
            error!("Could not exit: {}", e);
        }
    });
}

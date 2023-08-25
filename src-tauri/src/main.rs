// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tor;

use log::{debug, error, info, warn};
use tauri::{async_runtime, InvokePayload};
use tauri::{async_runtime::block_on, Manager, WindowEvent};
use tauri_plugin_log::LogTarget;
use tor::consts::setup_channels;
use tor::manager;

use crate::tor::check::tor_check;

fn main() {
    block_on(setup_channels());

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![tor_check])
        .on_window_event(|event| {
            let window = event.window();
            let windows = window.windows();

            if windows.len() > 1 {
                return;
            }

            match event.event() {
                WindowEvent::Destroyed => {
                    info!("Exiting...");
                    let res = block_on(manager::wait_and_stop_tor());
                    if res.is_err() {
                        error!("Could not stop tor: {}", res.unwrap_err());
                    }
                }
                _ => {}
            }
        })
        .setup(|app| {
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
                        splashscreen_window.close().unwrap();
                        window.show().unwrap();
                    }

                    if res.is_err() {
                        let err: anyhow::Error = res.unwrap_err();
                        window.close().unwrap();

                        error!("Could not start tor: {}", err);

                        splashscreen_window
                            .app_handle()
                            .emit_all("tor_start_error", err.to_string())
                            .unwrap();
                    }
                });
            });
            return Ok(());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

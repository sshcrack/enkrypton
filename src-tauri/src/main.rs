// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod payloads;
mod startup;
mod tor;
mod webserver;

use log::{error, info};
use startup::startup;
use tauri::{async_runtime::block_on, Manager, WindowEvent};
use tauri_plugin_log::LogTarget;
use tor::consts::setup_channels;
use tor::manager;
use webserver::server::start_webserver;

use crate::commands::restart;
use crate::commands::tor::tor_check;

fn main() {
    block_on(setup_channels());
    start_webserver();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![tor_check, restart])
        .on_window_event(|event| {
            let window = event.window();
            let windows = window.windows();

            if windows.len() != 0 {
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
            startup(app);
            return Ok(());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

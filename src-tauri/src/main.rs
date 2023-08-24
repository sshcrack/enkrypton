// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tor;


use log::{info, error};
use tauri::{async_runtime::block_on, Manager, WindowEvent};
use tauri_plugin_log::LogTarget;
use tor::misc::commands::*;
use tor::consts::setup_channels;
use tor::manager;

fn main() {
    block_on(setup_channels());

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![tor_start])
        .on_window_event(|event| match event.event() {
            WindowEvent::Destroyed => {
                info!("Exiting...");
                let res = block_on(manager::wait_and_stop_tor());
                if res.is_err() {
                    error!("Could not stop tor: {}", res.unwrap_err());
                }
            },
            _ => {}
        })
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            return Ok(());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

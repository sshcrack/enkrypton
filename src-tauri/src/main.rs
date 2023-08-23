// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tor;


use log::info;
use tauri::{async_runtime::block_on, Manager, WindowEvent};
use tauri_plugin_log::LogTarget;
use tor::commands::*;
use tor::manager;

fn main() {
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
                block_on(manager::stop_tor());
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

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    not(any(debug_assertions, feature = "enable-console")),
    windows_subsystem = "windows"
)]

mod commands;
mod startup;
mod util;

use log::{LevelFilter, error, info};

use commands::ws::*;
use messaging::server::server::start_webserver;
use shared::get_root_dir;
use startup::startup;
use tauri::{Manager, WindowEvent, async_runtime::block_on};
use tauri_plugin_log::Target;
use tauri_plugin_log::TargetKind;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tor_proxy::consts::setup_tor_channels;

use crate::commands::restart;
use crate::commands::storage::*;
use crate::commands::tor::*;
use crate::util::on_exit;

//noinspection RsCompileErrorMacro
/// This is the main loop of this application, basically does everything
fn main() {
    block_on(setup_tor_channels());
    // Start the local server which is used for receiving / sending messages to clients
    start_webserver();

    // Builds the Application and sets the logging level to debug
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    Target::new(TargetKind::Folder {
                        path: get_root_dir(),
                        file_name: None,
                    }),
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .with_colors(ColoredLevelConfig::default())
                .level_for("tauri", LevelFilter::Info)
                .level_for("hyper", LevelFilter::Info)
                .level(LevelFilter::Debug)
                .build(),
        )
        // Registering all commands
        .invoke_handler(tauri::generate_handler![
            restart,
            tor_check,
            tor_hostname,
            tor_is_alive,
            ws_connect,
            ws_send,
            storage_exists,
            storage_is_unlocked,
            storage_unlock_or_create,
            storage_delete,
            storage_set,
            storage_get,
            storage_save,
            splashscreen_closed
        ])
        // Closes the tor process when the application is closing
        .on_window_event(|window, event| {
            let windows = window.webview_windows();

            if windows.len() != 0 {
                return;
            }

            match event {
                WindowEvent::Destroyed => {
                    info!("Exiting...");
                    let res = block_on(on_exit());
                    if res.is_err() {
                        error!("Could not stop tor: {}", res.unwrap_err());
                    }
                }
                _ => {}
            }
        })
        // Starts the application and shows just the splashscreen for now.
        .setup(|app| {
            startup(app);
            return Ok(());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

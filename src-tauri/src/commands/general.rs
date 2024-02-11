// reserved for commands

use log::debug;
use sysinfo::System;
use tauri::Runtime;
use tor_proxy::consts::TOR_BINARY_PATH;

use crate::util::on_exit;

/// Restarts this application
#[tauri::command]
pub async fn restart<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    kill_old_tor: bool,
) -> Result<(), String> {
    // Kill tor if it's already running so there is no overlapping
    //TODO maybe find a better method by checking if process' binary is the same as the one we want to start
    if kill_old_tor {
        let tor_path = TOR_BINARY_PATH.file_name().unwrap();
        let tor_path = tor_path.to_str().unwrap();

        let s = System::new_all();
        let p = s.processes_by_exact_name(tor_path);

        for process in p {
            debug!(
                "{:?} {:?}",
                process.exe(),
                TOR_BINARY_PATH.to_str().unwrap()
            );
            if process.exe().is_some_and(|e| e == *TOR_BINARY_PATH) {
                process.kill();
            }
        }
    }

    on_exit().await.or_else(|e| Err(e.to_string()))?;

    app.restart();
    Ok(())
}

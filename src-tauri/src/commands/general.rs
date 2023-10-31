// reserved for commands

use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{async_runtime::block_on, Runtime};

use crate::{tor::consts::TOR_ZIP_PATH, util::on_exit};

/// Restarts this application
#[tauri::command]
pub fn restart<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<(), String> {
    block_on(on_exit()).or_else(|e| Err(e.to_string()))?;

    let tor_path = TOR_ZIP_PATH.file_name().unwrap();
    let tor_path = tor_path.to_str().unwrap();

    let s = System::new_all();
    let p = s.processes_by_exact_name(tor_path);

    for process in p {
        println!("{:?} {:?}", process.exe(), TOR_ZIP_PATH.to_str().unwrap());
        if process.exe() == *TOR_ZIP_PATH {
            process.kill();
        }
    }

    app.restart();
    Ok(())
}

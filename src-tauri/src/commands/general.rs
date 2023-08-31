// reserved for commands

use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{async_runtime::block_on, Runtime};

use crate::tor::{manager::wait_and_stop_tor, consts::TOR_BINARY_PATH};

#[tauri::command]
pub fn restart<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<(), String> {
    block_on(wait_and_stop_tor()).or_else(|e| Err(e.to_string()))?;

    let tor_path = TOR_BINARY_PATH.file_name().unwrap();
    let tor_path = tor_path.to_str().unwrap();

    let s = System::new_all();
    let p = s.processes_by_exact_name(tor_path);

    for process in p {
        println!("{:?} {:?}", process.exe(), TOR_BINARY_PATH.to_str().unwrap());
        if process.exe() == *TOR_BINARY_PATH {
            process.kill();
        }
    }

    app.restart();
    Ok(())
}

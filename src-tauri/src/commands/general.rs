// reserved for commands

use tauri::{async_runtime::block_on, Runtime};

use crate::tor::manager::wait_and_stop_tor;

#[tauri::command]
pub fn restart<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<(), String> {
    block_on(wait_and_stop_tor()).unwrap();
    app.restart();
    Ok(())
}

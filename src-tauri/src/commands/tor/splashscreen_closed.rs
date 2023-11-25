use tauri::{Runtime, Manager};

/// Returns wether the splashscreen is closed
#[tauri::command]
pub async fn splashscreen_closed<R: Runtime>(handle: tauri::AppHandle<R>) -> Result<bool, String> {
    let e = handle.get_window("splashscreen");
    if e.is_none() {
        return Ok(true)
    }

    let e = e.unwrap();
    if let Ok(x) = e.is_visible() {
        if x {
            return Ok(false)
        }
    }

    return Err("Invalid state".to_string())
}

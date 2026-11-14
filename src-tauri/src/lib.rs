#[tauri::command]
fn monitor_count(window: tauri::WebviewWindow) -> Result<usize, String> {
    Ok(window.available_monitors().map_err(|e| e.to_string())?.len())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![monitor_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
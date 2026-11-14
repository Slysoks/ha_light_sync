use std::{thread, time::Duration};

use serde::Serialize;
use tauri::{Emitter, Manager};

#[derive(Debug, PartialEq, Serialize)]
struct MonitorSnapshot {
    name: Option<String>,
    position: (i32, i32),
    size: (u32, u32),
    scale_factor: f64,
}

fn monitor_snapshot(app: &tauri::AppHandle) -> Result<Vec<MonitorSnapshot>, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    let mut monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();

            MonitorSnapshot {
                name: monitor.name().map(ToOwned::to_owned),
                position: (position.x, position.y),
                size: (size.width, size.height),
                scale_factor: monitor.scale_factor(),
            }
        })
        .collect::<Vec<_>>();

    monitors.sort_by_key(|monitor| monitor.position);
    Ok(monitors)
}

#[tauri::command]
fn monitor_count(window: tauri::WebviewWindow) -> Result<usize, String> {
    Ok(window.available_monitors().map_err(|e| e.to_string())?.len())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![monitor_count])
        .setup(|app| {
            let app_handle = app.handle().clone();

            thread::spawn(move || {
                let mut previous = match monitor_snapshot(&app_handle) {
                    Ok(monitors) => monitors,
                    Err(error) => {
                        eprintln!("Unable to read monitors: {error}");
                        Vec::new()
                    }
                };

                loop {
                    thread::sleep(Duration::from_secs(1));

                    let current = match monitor_snapshot(&app_handle) {
                        Ok(monitors) => monitors,
                        Err(error) => {
                            eprintln!("Unable to read monitors: {error}");
                            continue;
                        }
                    };

                    if current != previous {
                        if let Err(error) =
                            app_handle.emit("monitors-changed", current.len())
                        {
                            eprintln!("Unable to notify frontend about monitor change: {error}");
                        }
                        previous = current;
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
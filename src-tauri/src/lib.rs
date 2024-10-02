// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg(desktop)]
mod tray;

use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{window, CloseRequestApi, Emitter, Manager, WebviewWindowBuilder}; // Import WindowUrl
use tauri::{AppHandle, WindowEvent};
use tempfile::TempDir; // Add this import

mod mouse_monitor;

#[tauri::command]
async fn handle_file_drop(
    app: tauri::AppHandle, // Prefix with underscore to silence warning
    paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut stored_paths = Vec::new();

    for path in paths {
        let source_path = PathBuf::from(path);
        if source_path.is_dir() {
            // Handle directory: iterate through its contents
            for entry in fs::read_dir(&source_path).map_err(|e| e.to_string())? {
                match entry {
                    Ok(entry) => {
                        // Store the source path of the file
                        stored_paths.push(entry.path().to_string_lossy().into_owned());
                    }
                    Err(e) => eprintln!("Error reading entry: {}", e), // Log the error
                }
            }
        } else {
            // Handle individual file
            stored_paths.push(source_path.to_string_lossy().into_owned());
        }
    }
    app.emit("files_dropped", ()).map_err(|e| e.to_string())?;

    // Show the window after handling the file drop
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
    }

    Ok(stored_paths)
}

#[tauri::command]
async fn handle_browser_file_drop(
    app: tauri::AppHandle,
    files: Vec<String>,
) -> Result<Vec<String>, String> {
    let dest_dir = app
        .path()
        .app_local_data_dir()
        .unwrap()
        .join("uploaded_files");
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let mut stored_paths = Vec::new();

    for file in files.iter() {
        if let Ok(path) = PathBuf::from_str(&file) {
            // Local file drop (path provided)
            if path.exists() {
                println!("Local file drop: {}", path.display());
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let dest_path = dest_dir.join(file_name);
                fs::copy(&path, &dest_path).map_err(|e| e.to_string())?;
                stored_paths.push(dest_path.to_string_lossy().into_owned());
            } else {
                // Browser file drop (file data provided)
                if let Some(data_uri) = file.strip_prefix("data:") {
                    let parts: Vec<&str> = data_uri.splitn(2, ',').collect();
                    if parts.len() == 2 {
                        let (header, data) = (parts[0], parts[1]);
                        let file_name = header
                            .split(';')
                            .find(|&s| s.starts_with("name="))
                            .and_then(|s| s.split('=').nth(1))
                            .unwrap_or("unnamed_file");

                        let file_path = dest_dir.join(file_name);

                        let decoded = general_purpose::STANDARD
                            .decode(data)
                            .map_err(|e| e.to_string())?;
                        fs::write(&file_path, decoded).map_err(|e| e.to_string())?;

                        println!("Browser file drop saved: {}", file_path.display());
                        stored_paths.push(file_path.to_string_lossy().into_owned());
                    } else {
                        return Err("Invalid data URI format".to_string());
                    }
                } else {
                    return Err("Invalid file data format".to_string());
                }
            }
        } else {
            return Err("Invalid file path or data".to_string());
        }
    }

    Ok(stored_paths)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn copy_selected_files(app: tauri::AppHandle, paths: Vec<String>) -> Result<(), String> {
    let dest_dir = app
        .path()
        .app_local_data_dir()
        .unwrap()
        .join("uploaded_files");
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    for path in paths {
        let source_path = PathBuf::from(&path);
        if let Some(file_name) = source_path.file_name() {
            let dest_path = dest_dir.join(file_name);
            fs::copy(&source_path, &dest_path).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            handle_file_drop,
            handle_browser_file_drop,
            copy_selected_files
        ])
        .setup(|app| {
            // Create a TempDir during setup and store it in the app state
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            app.manage(temp_dir);
            #[cfg(all(desktop))]
            {
                let handle = app.handle();
                tray::create_tray(handle)?; // Create system tray
            }
            // Correctly call start_mouse_monitor with a closure
            mouse_monitor::start_mouse_monitor(4, 500, app.handle().clone()); // Changed from app to app.handle()

            Ok(())
        })
        .on_window_event(|_window, _event| match _event {
            WindowEvent::CloseRequested { api, .. } => {
                _window.hide().unwrap();
                api.prevent_close()
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

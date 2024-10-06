use std::thread;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};
use tauri::{Listener, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::file::FileMetadata;
use crate::FileList;

use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use windows_icons::{get_icon_base64_by_path, get_icon_by_path};

#[tauri::command]
pub fn add_files(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
    files: Vec<String>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    for (index, path_str) in files.iter().enumerate() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let metadata = path.metadata().map_err(|e| e.to_string())?;
            let file = FileMetadata {
                id: list.len() as u64 + index as u64 + 1,
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                path: path.clone(),
                size: metadata.len(),
                file_type: path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            };
            // Avoid duplicates
            if !list.iter().any(|f| f.path == file.path) {
                list.push(file);
            }
            let _ = app_handle.emit("file_added", "test");
        } else {
            println!("File does not exist: {}", path_str);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn remove_file(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
    file_id: u64,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    if let Some(pos) = list.iter().position(|f| f.id == file_id) {
        list.remove(pos);
        println!("Removed file with ID: {}", file_id);

        app_handle.emit("file_removed", "test").unwrap();
        Ok(())
    } else {
        Err(format!("File with ID {} not found", file_id))
    }
}

#[tauri::command]
pub fn get_files(file_list: State<'_, FileList>) -> Result<Vec<FileMetadata>, String> {
    let list: std::sync::MutexGuard<'_, Vec<FileMetadata>> = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    Ok(list.clone())
}

#[tauri::command]
pub fn rename_file(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
    file_id: u64,
    new_name: String,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    if let Some(file) = list.iter_mut().find(|f| f.id == file_id) {
        file.name = new_name.clone();
        println!("Renamed file ID {} to {}", file_id, new_name);
        app_handle.emit("file_renamed", "file").unwrap();
        Ok(())
    } else {
        Err(format!("File with ID {} not found", file_id))
    }
}

#[tauri::command]
pub fn start_drag(
    app: tauri::AppHandle,
    file_path: String,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    println!("Starting drag for file: {}", file_path);
    let item = match std::fs::canonicalize(file_path.clone()) {
        Ok(path) => {
            if !path.exists() {
                return Err(format!("File not found: {}", file_path));
            }
            drag::DragItem::Files(vec![path])
        }
        Err(e) => {
            return Err(format!("Error finding file: {} ({})", file_path, e));
        }
    };

    let window = app.get_webview_window("main").unwrap();
    // Define the on_drop_callback function
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Drag result: {:?}", result);
    };

    // Start the drag operation
    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .expect("Failed to start drag operation");

    println!("Starting drag for file: {}", file_path);
    Ok(())
}
#[tauri::command]
pub fn clear_files(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    // Clear all files from the list
    list.clear();

    println!("Cleared all files");

    // Emit an event to notify the frontend that all files have been cleared
    app_handle
        .emit("files_updated", ())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn start_multi_drag(
    app: tauri::AppHandle,
    file_paths: Vec<String>,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

    let mut valid_paths = Vec::new();

    for file_path in file_paths {
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    valid_paths.push(path);
                } else {
                    println!("File not found: {}", file_path);
                }
            }
            Err(e) => {
                println!("Error finding file: {} ({})", file_path, e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    let item = drag::DragItem::Files(valid_paths);

    let window = app.get_webview_window("main").unwrap();
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Multi-file drag result: {:?}", result);
    };

    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .map_err(|e| format!("Failed to start multi-file drag operation: {}", e))?;

    Ok(())
}

// Define a struct for mouse monitor configuration

#[tauri::command]
pub fn open_popup_window(
    app: tauri::AppHandle,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    // Get the main window
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get the position and size of the main window
    let position = main_window.outer_position().map_err(|e| e.to_string())?;
    let size = main_window.outer_size().map_err(|e| e.to_string())?;

    // Define popup window dimensions
    let popup_width = 450.0;
    let popup_height = 350.0;

    // Calculate the position for the popup window (centered below the main window)
    let popup_x = position.x as f64 + (size.width as f64 - popup_width) / 2.0;
    let popup_y = position.y as f64 + size.height as f64 + 5.0;

    // Create the popup window
    tauri::async_runtime::spawn(async move {
        WebviewWindowBuilder::new(
            &app,
            "popup",                         // Window label
            WebviewUrl::App("popup".into()), // Assuming same frontend build
        )
        .title("File List")
        .decorations(false) // Remove window decorations for a popup feel
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .inner_size(popup_width, popup_height)
        .position(popup_x, popup_y)
        .always_on_top(true)
        .focused(false)
        .build()
        .map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    });

    Ok(())
}

#[tauri::command]
pub fn close_popup_window(
    app: tauri::AppHandle,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    let popup_window = app
        .get_webview_window("popup")
        .ok_or("Popup window not found")?;
    popup_window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_file_icon_base64(file_path: &str) -> Result<String, String> {
    Ok(get_icon_base64_by_path(file_path))
}

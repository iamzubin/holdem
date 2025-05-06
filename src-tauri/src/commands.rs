use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::config::AppConfig;
use crate::file::FileMetadata;
use crate::FileList;

use windows_icons::get_icon_base64_by_path;

#[tauri::command]
pub fn add_files(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
    files: Vec<String>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    for (_index, path_str) in files.iter().enumerate() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let metadata = path.metadata().map_err(|e| e.to_string())?;
            let file = FileMetadata {
                id: list.len() as u64,
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
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn remove_files(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
    file_ids: Vec<u64>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    for file_id in file_ids {
        if let Some(pos) = list.iter().position(|f| f.id == file_id) {
            list.remove(pos);
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        } else {
            return Err(format!("File with ID {} not found", file_id));
        }
    }
    Ok(())
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
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err(format!("File with ID {} not found", file_id))
    }
}

#[tauri::command]
pub fn start_drag(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
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

    let window = app.get_webview_window("main").unwrap().clone();
    // Define the on_drop_callback function
    let on_drop_callback = move |_, _| {
        println!("drop callback");
    };

    // Start the drag operation
    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .expect("Failed to start drag operation");

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

    // Emit an event to notify the frontend that all files have been cleared
    app_handle
        .emit("files_updated", ())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn start_multi_drag(
    app: tauri::AppHandle,
    _file_list: State<'_, FileList>,
    file_paths: Vec<String>,
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
                }
            }
            Err(e) => {
                println!("Error processing file: {}", e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    let item = drag::DragItem::Files(valid_paths);

    let window = app.get_webview_window("main").unwrap();

    let app_clone = app.clone();
    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        // check if the file is dropped on the app window
        if matches!(result, drag::DragResult::Cancel) {
            return;
        }

        // check if the popup window is open
        if app_clone.get_webview_window("popup").is_some() {
            close_popup_window(app_clone.clone()).unwrap();
        }
        let _ = app
            .get_webview_window("main")
            .unwrap()
            .hide()
            .map_err(|e| e.to_string());
        // Emit an event to notify the frontend that all files have been cleared
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
pub fn open_popup_window(app: tauri::AppHandle) -> Result<(), String> {
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

    if let Some(popup_window) = app.get_webview_window("popup") {
        popup_window.close().map_err(|e| e.to_string())?;
    } else {
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
    }
    Ok(())
}

#[tauri::command]
pub fn close_popup_window(app: tauri::AppHandle) -> Result<(), String> {
    let popup_window = app
        .get_webview_window("popup")
        .ok_or("Popup window not found")?;
    popup_window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_file_icon_base64(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
    file_path: &str,
) -> Result<String, String> {
    let file_path = file_path.to_string();
    if !std::path::Path::new(&file_path).exists() {
        // Remove the file from the list if it no longer exists
        let mut list = file_list
            .lock()
            .map_err(|_| "Failed to acquire lock".to_string())?;
        if let Some(pos) = list
            .iter()
            .position(|f| f.path.to_string_lossy() == file_path)
        {
            list.remove(pos);
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        }
        return Ok("".to_string()); // Return empty string for non-existent files
    }
    Ok(get_icon_base64_by_path(&file_path))
}

#[tauri::command]
pub fn refresh_file_list(
    app_handle: tauri::AppHandle,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    let mut needs_update = false;

    // Create a new list to store valid files
    let mut new_list = Vec::new();

    for file in list.iter() {
        if file.path.exists() {
            new_list.push(file.clone());
        } else {
            needs_update = true;
        }
    }

    if needs_update {
        *list = new_list;
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_config(config: State<Arc<Mutex<AppConfig>>>) -> AppConfig {
    config.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_config(
    new_config: AppConfig,
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = config.lock().unwrap();
    *config = new_config;
    config.save(&app_handle)
}

#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    // Get the main window
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get the position and size of the main window
    let position = main_window.outer_position().map_err(|e| e.to_string())?;
    let size = main_window.outer_size().map_err(|e| e.to_string())?;

    // Define settings window dimensions
    let settings_width = 500.0;
    let settings_height = 600.0;

    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.close().map_err(|e| e.to_string())?;
    } else {
        // Create the settings window
        tauri::async_runtime::spawn(async move {
            WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings".into()))
                .title("Settings")
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .inner_size(settings_width, settings_height)
                .focused(true)
                .build()
                .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        });
    }
    Ok(())
}

#[tauri::command]
pub fn close_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    app.restart();
    Ok(())
}

#[tauri::command]
pub fn set_autostart(app_handle: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    if enabled {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn register_hotkey(app_handle: tauri::AppHandle, shortcut_str: String) -> Result<(), String> {
    if shortcut_str.is_empty() {
        return Ok(());
    }

    let app_handle_clone = app_handle.clone();
    println!("Registering hotkey: {}", shortcut_str);

    // Parse the shortcut string
    let mut modifiers = Modifiers::empty();
    let mut code = Code::KeyN; // Default key

    for part in shortcut_str.split('+') {
        let part = part.trim();
        println!("part: {}", part);
        match part.to_uppercase().as_str() {
            "CTRL" | "Ctrl" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" | "Shift" => modifiers |= Modifiers::SHIFT,
            "ALT" | "Alt" => modifiers |= Modifiers::ALT,
            "META" | "Meta" | "WIN" | "Win" | "COMMAND" => modifiers |= Modifiers::META,
            key => {
                // Handle letter keys
                if key.len() == 1 && key.chars().next().unwrap().is_alphabetic() {
                    code = match key {
                        "A" => Code::KeyA,
                        "B" => Code::KeyB,
                        "C" => Code::KeyC,
                        "D" => Code::KeyD,
                        "E" => Code::KeyE,
                        "F" => Code::KeyF,
                        "G" => Code::KeyG,
                        "H" => Code::KeyH,
                        "I" => Code::KeyI,
                        "J" => Code::KeyJ,
                        "K" => Code::KeyK,
                        "L" => Code::KeyL,
                        "M" => Code::KeyM,
                        "N" => Code::KeyN,
                        "O" => Code::KeyO,
                        "P" => Code::KeyP,
                        "Q" => Code::KeyQ,
                        "R" => Code::KeyR,
                        "S" => Code::KeyS,
                        "T" => Code::KeyT,
                        "U" => Code::KeyU,
                        "V" => Code::KeyV,
                        "W" => Code::KeyW,
                        "X" => Code::KeyX,
                        "Y" => Code::KeyY,
                        "Z" => Code::KeyZ,
                        _ => Code::KeyN,
                    };
                } else {
                    // Try to parse other keys
                    if let Ok(k) = key.parse::<Code>() {
                        code = k;
                    } else {
                        println!("Failed to parse key: {}", key);
                    }
                }
            }
        }
    }

    println!("Final code: {:?}", code);
    println!("Final modifiers: {:?}", modifiers);

    let shortcut = Shortcut::new(Some(modifiers), code);
    println!("Created shortcut: {:?}", shortcut);

    // First unregister all hotkeys
    if let Err(e) = app_handle.global_shortcut().unregister_all() {
        println!("Failed to unregister all hotkeys: {}", e);
    }

    // Register the shortcut
    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    // Set up the callback to show the window
    app_handle
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
            println!("Shortcut pressed");
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                // Show the window first
                if let Err(e) = window.show() {
                    println!("Failed to show window: {}", e);
                    return;
                }

                // Then try to focus it
                if let Err(e) = window.set_focus() {
                    println!("Failed to focus window: {}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to set shortcut callback: {}", e))?;

    println!("Hotkey registered successfully");
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

use commands::{
    add_files, clear_files, close_popup_window, close_settings_window, get_config,
    get_file_icon_base64, get_files, open_popup_window, open_settings_window, refresh_file_list,
    register_hotkey, remove_files, rename_file, restart_app, save_config, set_autostart,
    show_main_window, start_drag, start_multi_drag,
};
use config::AppConfig;
use file::get_dir_size;
use file::FileMetadata;
use mouse_monitor::start_mouse_monitor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Listener;
use tauri::Manager;
use tauri::PhysicalPosition;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use windows::Win32::Foundation::POINT;
use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;
use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod commands;
#[cfg(desktop)]
mod tray;

mod config;
mod file; // Make sure to include the file module
mod mouse_monitor;
type FileList = Arc<Mutex<Vec<FileMetadata>>>;

fn handle_file_drop(event: tauri::Event, file_list: FileList, app_handle: tauri::AppHandle) {
    let payload: serde_json::Value = serde_json::from_str(event.payload()).unwrap_or_default();
    
    // Handle text drops
    if let Some(text) = payload["text"].as_str() {
        let temp_dir = std::env::temp_dir();
        let timestamp = chrono::Local::now();
        let folder_name = timestamp.format("%Y%m%d").to_string();
        let drop_folder = temp_dir.join("holdem_drops").join(&folder_name);
        std::fs::create_dir_all(&drop_folder).ok();
        
        let file_name = format!("dropped_text_{}.txt", timestamp.format("%H%M%S"));
        let file_path = drop_folder.join(&file_name);
        
        if let Ok(_) = std::fs::write(&file_path, text) {
            let mut list = file_list.lock().unwrap();
            let file = FileMetadata {
                id: list.len() as u64,
                name: file_name,
                path: file_path.clone(),
                size: text.len() as u64,
                file_type: "txt".to_string(),
            };
            if !list.iter().any(|f| f.path == file.path) {
                println!("text file added: {:?}", file);
                list.push(file);
            }
            drop(list);
            if let Err(e) = app_handle.emit("files_updated", ()) {
                eprintln!("Failed to emit files_updated event: {}", e);
            }
        }
        return;
    }

    let files: Vec<String> = payload["paths"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut list = file_list.lock().unwrap();

    for path_str in files.iter() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            if let Ok(metadata) = path.metadata() {
                // If file is in temp directory, copy it to a permanent location
                let final_path = if path.starts_with(std::env::temp_dir()) {
                    let timestamp = chrono::Local::now();
                    let folder_name = timestamp.format("%Y%m%d").to_string();
                    let drop_folder = std::env::temp_dir().join("holdem_drops").join(&folder_name);
                    std::fs::create_dir_all(&drop_folder).ok();
                    
                    let file_name = path.file_name().unwrap_or_default();
                    let new_path = drop_folder.join(file_name);
                    if let Ok(_) = std::fs::copy(&path, &new_path) {
                        new_path
                    } else {
                        path.clone()
                    }
                } else {
                    path.clone()
                };

                // Calculate size correctly for directories
                let size = if metadata.is_dir() {
                    get_dir_size(&path).unwrap_or(0)
                } else {
                    metadata.len()
                };

                let file = FileMetadata {
                    id: list.len() as u64,
                    name: final_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    path: final_path,
                    size,
                    file_type: if metadata.is_dir() {
                        "folder".to_string()
                    } else {
                        path.extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    },
                };
                // Avoid duplicates
                if !list.iter().any(|f| f.path == file.path) {
                    println!("file added: {:?}", file);
                    list.push(file);
                }
            }
        }
    }

    // Drop the lock before emitting the event
    drop(list);
    if let Err(e) = app_handle.emit("files_updated", ()) {
        eprintln!("Failed to emit files_updated event: {}", e);
    }

    // Cleanup old files
    cleanup_old_files();
}

fn cleanup_old_files() {
    let temp_dir = std::env::temp_dir().join("holdem_drops");
    if let Ok(entries) = std::fs::read_dir(temp_dir) {
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Some(folder_name) = entry.file_name().to_str() {
                        // If the folder is not from today, delete it
                        if folder_name != today {
                            let _ = std::fs::remove_dir_all(entry.path());
                        }
                    }
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Check for existing instance
    unsafe {
        let mutex_name = windows::core::w!("Global\\HoldemAppMutex");
        let mutex = CreateMutexW(None, true, mutex_name);

        if let Ok(mutex) = mutex {
            if WaitForSingleObject(mutex, 0) == WAIT_OBJECT_0 {
                tauri::Builder::default()
                    .plugin(tauri_plugin_process::init())
                    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
                    .plugin(tauri_plugin_fs::init())
                    .plugin(tauri_plugin_shell::init())
                    .plugin(tauri_plugin_updater::Builder::new().build())
                    .plugin(tauri_plugin_dialog::init())
                    .plugin(tauri_plugin_autostart::Builder::new().build())
                    .plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_handler(move |app, shortcut, event| {
                                println!("Hotkey pressed: {:?}", shortcut);
                                if let Some(window) = app.get_webview_window("main") {
                                    match event.state() {
                                        tauri_plugin_global_shortcut::ShortcutState::Pressed => {
                                            println!("Showing window");

                                            // Get current cursor position
                                            let mut cursor_pos = POINT { x: 0, y: 0 };
                                            let _ = GetCursorPos(&mut cursor_pos);

                                            // Define margin to consider as "corner"
                                            let margin = 200; // Same as shake_threshold in mouse monitor

                                            // Adjust position to avoid off-screen
                                            let mut window_x = cursor_pos.x;
                                            let mut window_y = cursor_pos.y;

                                            // Check if cursor is near the right or bottom edge
                                            if cursor_pos.x + margin
                                                > GetSystemMetrics(SM_CXSCREEN)
                                            {
                                                window_x = cursor_pos.x - margin;
                                            }

                                            if cursor_pos.y + margin
                                                > GetSystemMetrics(SM_CYSCREEN)
                                            {
                                                window_y = cursor_pos.y - margin;
                                            }

                                            // Set window position and show it
                                            let _ = window.set_position(PhysicalPosition {
                                                x: window_x,
                                                y: window_y,
                                            });
                                            let _ = window.show();
                                            let _ = window.unminimize();
                                            let _ = window.set_focus();
                                        }
                                        tauri_plugin_global_shortcut::ShortcutState::Released => {
                                            println!("Window shown");
                                        }
                                    }
                                }
                            })
                            .build(),
                    )
                    .invoke_handler(tauri::generate_handler![
                        start_drag,
                        start_multi_drag,
                        open_popup_window,
                        close_popup_window,
                        add_files,
                        remove_files,
                        get_files,
                        rename_file,
                        get_file_icon_base64,
                        clear_files,
                        refresh_file_list,
                        get_config,
                        save_config,
                        open_settings_window,
                        close_settings_window,
                        restart_app,
                        set_autostart,
                        register_hotkey,
                        show_main_window,
                    ])
                    .setup(|app| {
                        // Create file list here
                        let file_list: FileList = Arc::new(Mutex::new(Vec::new()));
                        app.manage(file_list.clone());

                        // Check for updates on startup
                        let app_handle = app.handle().clone();
                        tauri::async_runtime::spawn(async move {
                            if let Ok(updater) = app_handle.updater() {
                                if let Ok(Some(update)) = updater.check().await {
                                    let empty = String::new();
                                    let body = update.body.as_ref().unwrap_or(&empty);
                                    let _ = app_handle.dialog()
                                        .message(format!("Update to {} is available!\n\nRelease notes: {}", update.version, body))
                                        .title("Update Available")
                                        .kind(MessageDialogKind::Info)
                                        .blocking_show();
                                    
                                    let _ = update.download_and_install(|_, _| {}, || {}).await;
                                }
                            }
                        });

                        // Ensure config directory exists
                        let config_dir = app.handle().path().app_config_dir().unwrap();
                        println!("App config directory: {:?}", config_dir);
                        if !config_dir.exists() {
                            println!("Creating app config directory...");
                            std::fs::create_dir_all(&config_dir).map_err(|e| {
                                format!("Failed to create app config directory: {}", e)
                            })?;
                        }

                        // Load configuration
                        let config = AppConfig::load(&app.handle());
                        app.manage(Arc::new(Mutex::new(config.clone())));

                        // Register hotkey if configured
                        if !config.hotkey.is_empty() {
                            println!("Registering startup hotkey: {}", config.hotkey);
                            // Wait a bit before registering the hotkey
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            if let Err(e) =
                                register_hotkey(app.handle().clone(), config.hotkey.clone())
                            {
                                println!("Failed to register startup hotkey: {}", e);
                            } else {
                                println!("Successfully registered startup hotkey");
                            }
                        }

                        #[cfg(all(desktop))]
                        {
                            let handle = app.handle();
                            tray::create_tray(handle)?;
                        }

                        // Start the mouse monitor with configuration
                        let app_handle = app.handle().clone();
                        let config = app.state::<Arc<Mutex<AppConfig>>>();
                        let config = config.lock().unwrap();
                        start_mouse_monitor(config.mouse_monitor.clone(), app_handle.clone());

                        let file_list_clone = file_list.clone();
                        let app_handle = app.handle().clone();

                        app.listen_any("tauri://drag-drop", move |event| {
                            println!("file dropped event received in setup");
                            handle_file_drop(event, file_list_clone.clone(), app_handle.clone());
                        });

                        Ok(())
                    })
                    .run(tauri::generate_context!())
                    .expect("error while running tauri application");

                // Clean up the mutex
                let _ = ReleaseMutex(mutex);
            } else {
                // Another instance is already running
                println!("Another instance of the application is already running.");
                std::process::exit(0);
            }
        }
    }
}

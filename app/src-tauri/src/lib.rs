use std::sync::Arc;
use std::sync::Mutex;
use tauri::{Manager, Listener};
use tauri::PhysicalPosition;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::DialogExt;
use windows::Win32::Foundation::POINT;
use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;
use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;
use tauri::WebviewUrl;
use tauri::WebviewWindowBuilder;

mod commands;
#[cfg(desktop)]
mod tray;
mod config;
mod file;
mod mouse_monitor;
mod file_drop;

use commands::{
    file_ops::*,
    window_ops::*,
    drag_ops::*,
    config_ops::*,
};
use config::AppConfig;
use file::FileMetadata;
use mouse_monitor::start_mouse_monitor;

type FileList = Arc<Mutex<Vec<FileMetadata>>>;

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
                    .plugin(
                        tauri_plugin_autostart::Builder::new()
                            .args(vec!["--autostart"])
                            .build()
                    )
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
                                            if cursor_pos.x + margin > GetSystemMetrics(SM_CXSCREEN) {
                                                window_x = cursor_pos.x - margin;
                                            }

                                            if cursor_pos.y + margin > GetSystemMetrics(SM_CYSCREEN) {
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
                                if let Ok(Some(_update)) = updater.check().await {
                                    // Open the updater window if an update is available
                                    if let Some(existing_window) = app_handle.get_webview_window("updater") {
                                        let _ = existing_window.show();
                                        let _ = existing_window.set_focus();
                                    } else {
                                        let _ = WebviewWindowBuilder::new(
                                            &app_handle,
                                            "updater",
                                            WebviewUrl::App("/updater".into())
                                        )
                                        .title("Software Updates")
                                        .inner_size(500.0, 400.0)
                                        .center()
                                        .decorations(false)
                                        .build();
                                    }
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
                            if let Err(e) = register_hotkey(app.handle().clone(), config.hotkey.clone()) {
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
                            file_drop::handle_file_drop(event, file_list_clone.clone(), app_handle.clone());
                        });

                        let is_autostart = std::env::args().any(|arg| arg == "--autostart");
                        if is_autostart {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        } else {
                            // Normal launch, show the main window
                        }

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

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, Listener, PhysicalPosition};

#[derive(Clone)]
struct DragState {
    drag_started: Arc<AtomicBool>,
    successful_drop: Arc<AtomicBool>,
}
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{POINT, WAIT_OBJECT_0};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{CreateMutexW, WaitForSingleObject, ReleaseMutex};
use tauri_plugin_updater::UpdaterExt;
use tauri::WebviewUrl;
use tauri::WebviewWindowBuilder;
use tauri::{DragDropEvent, WindowEvent};
mod commands;
#[cfg(desktop)]
mod tray;
mod config;
mod file;
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod mouse_monitor;
mod file_drop;
mod thumbnail;
mod analytics;
mod logging;
mod utils;

use commands::{
    file_ops::*,
    window_ops::*,
    drag_ops::*,
    config_ops::*,
};
use config::AppConfig;
use file::FileMetadata;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use mouse_monitor::start_mouse_monitor;
use analytics::AnalyticsService;




type FileList = Arc<Mutex<Vec<FileMetadata>>>;

fn build_app() -> tauri::Builder<tauri::Wry> {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(vec!["--autostart"])
                .build()
        );

    #[cfg(target_os = "windows")]
    {
        builder = builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    println!("Hotkey pressed: {:?}", shortcut);
                    if let Some(window) = app.get_webview_window("main") {
                        match event.state() {
                            tauri_plugin_global_shortcut::ShortcutState::Pressed => {
                                println!("Showing window");

                                let mut cursor_pos = POINT { x: 0, y: 0 };
                                let _ = unsafe { GetCursorPos(&mut cursor_pos) };
                                let margin = 200.0;

                                if let Ok(bounds) = crate::utils::ScreenBounds::from_window(&window) {
                                    let (window_x, window_y) = bounds.constrain_position(
                                        cursor_pos.x as f64,
                                        cursor_pos.y as f64,
                                        margin,
                                    );
                                    let _ = window.set_position(PhysicalPosition {
                                        x: window_x as i32,
                                        y: window_y as i32,
                                    });
                                }

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
        );
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_key_intercept::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            // start_drag,
            start_multi_drag,
            open_popup_window,
            close_popup_window,
            open_consent_window,
            close_consent_window,
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
            accept_analytics_consent,
            decline_analytics_consent,
            check_analytics_consent,
            check_config_exists,
            check_input_monitoring_permission,
            open_input_monitoring_settings,
        ])
        .setup(|app| {
            // Ensure config directory exists first
            let config_dir = app.handle().path().app_config_dir().unwrap_or_else(|_| {
                eprintln!("Failed to get app config directory");
                std::process::exit(1);
            });
            println!("App config directory: {:?}", config_dir);
            if !config_dir.exists() {
                println!("Creating app config directory...");
                if let Err(e) = std::fs::create_dir_all(&config_dir) {
                    eprintln!("Failed to create app config directory: {}", e);
                    return Err(format!("Failed to create app config directory: {}", e).into());
                }
            }

            // Load configuration once
            let config = AppConfig::load(app.handle());
            app.manage(Arc::new(Mutex::new(config.clone())));

            // Initialize analytics service
            let analytics_service = AnalyticsService::new();
            let analytics_enabled = config.analytics_enabled;
            let uuid = config.analytics_uuid.clone();
            
            // Store analytics service in app state first
            let analytics_state = Arc::new(Mutex::new(analytics_service));
            app.manage(analytics_state.clone());
            
            // Initialize analytics service asynchronously
            let analytics_state_clone = analytics_state.clone();
            tauri::async_runtime::spawn(async move {
                let mut service = AnalyticsService::new();
                let _ = service.initialize(analytics_enabled, uuid).await;
                
                // Update the service with the initialized client
                if let Ok(mut state) = analytics_state_clone.lock() {
                    *state = service;
                }
            });

            // Send app_started event if analytics is enabled
            if config.analytics_enabled {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = analytics::send_analytics_event(&app_handle, "app_started", None).await;
                });
            }

            // Create file list here
            let file_list: FileList = Arc::new(Mutex::new(Vec::new()));
            app.manage(file_list.clone());

            // Create drag state
            let drag_state = Arc::new(DragState {
                drag_started: Arc::new(AtomicBool::new(false)),
                successful_drop: Arc::new(AtomicBool::new(false)),
            });
            app.manage(drag_state.clone());

            // Check for updates on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(updater) = app_handle.updater() {
                    if let Ok(Some(_update)) = updater.check().await {
                        // Send analytics event for update available (fire and forget)
                        std::mem::drop(analytics::send_update_checked_event(&app_handle, true));
                        
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
                    } else {
                        // Send analytics event for no update available (fire and forget)
                        std::mem::drop(analytics::send_update_checked_event(&app_handle, false));
                    }
                }
            });

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

            #[cfg(desktop)]
            {
                let handle = app.handle();
                tray::create_tray(handle)?;
            }

            // Start the mouse monitor with configuration (Windows and macOS)
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            {
                let app_handle = app.handle().clone();
                let config_state = app.state::<Arc<Mutex<AppConfig>>>();
                let config_guard = config_state.lock().map_err(|e| {
                    format!("Failed to lock config: {}", e)
                })?;
                start_mouse_monitor(config_guard.mouse_monitor.clone(), app_handle.clone(), drag_state.clone());
            }

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
        .on_window_event(|window, event| {
            if let WindowEvent::DragDrop(drop_event) = event {
                let drag_state = window.app_handle().state::<Arc<DragState>>().inner();
                match drop_event {
                    DragDropEvent::Enter { .. } => {
                        println!("Drag entered the window bounds");
                        drag_state.drag_started.store(true, Ordering::Relaxed);
                    }
                    DragDropEvent::Drop { paths, .. } => {
                        println!("Dropped cleanly in the app! Files: {:?}", paths);
                        // Handle the file drop
                        let app_handle = window.app_handle();
                        let file_list_state = app_handle.state::<FileList>();
                        file_drop::handle_file_drop_from_paths(paths.clone(), file_list_state.inner().clone(), app_handle.clone());
                        
                        // Do not hide the window after processing - let user interact with the files
                        drag_state.drag_started.store(false, Ordering::Relaxed);
                        drag_state.successful_drop.store(true, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::setup_logging();
    tracing::info!("Starting holdem application");

    #[cfg(target_os = "windows")]
    {
        // Check for existing instance
        unsafe {
            let mutex_name = windows::core::w!("Global\\HoldemAppMutex");
            let mutex = CreateMutexW(None, true, mutex_name);

            if let Ok(mutex) = mutex {
                if WaitForSingleObject(mutex, 0) == WAIT_OBJECT_0 {
                    build_app()
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

    #[cfg(not(target_os = "windows"))]
    {
        build_app()
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}

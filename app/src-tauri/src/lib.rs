use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;
use tracing::{error, info, warn};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

pub(crate) struct DragState {
    pub(crate) drag_started: AtomicBool,
    pub(crate) successful_drop: AtomicBool,
}

mod analytics;
mod commands;
mod config;
mod file;
mod drop_target;

mod file_drop;
mod logging;
mod mouse_monitor;
mod thumbnail;
#[cfg(desktop)]
mod tray;
mod utils;

use analytics::AnalyticsService;
use commands::{config_ops::*, drag_ops::*, file_ops::*, window_ops::*};
pub(crate) type FileList = Arc<Mutex<Vec<file::FileMetadata>>>;

fn build_app() -> tauri::Builder<tauri::Wry> {
    let mut builder = tauri::Builder::default()
        // Must be registered first. Launching Holdem again (Start menu,
        // shortcut) shows the running shelf instead of starting a second
        // copy. The lock is per user session, so other Windows users on the
        // same machine can run their own instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            info!("Second launch detected, showing the existing shelf");
            if let Err(e) = commands::window_ops::reveal_main_window(app) {
                warn!("Failed to show the existing shelf: {}", e);
            }
        }))
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(vec!["--autostart"])
                .build(),
        );

    builder = builder.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                info!("Global hotkey event received: {:?}", shortcut);
                if let Some(window) = app.get_webview_window("main") {
                    match event.state() {
                        tauri_plugin_global_shortcut::ShortcutState::Pressed => {
                            let mut cursor_pos = POINT { x: 0, y: 0 };
                            if unsafe { GetCursorPos(&mut cursor_pos) }.is_ok() {
                                crate::utils::move_window_to_cursor(
                                    &window,
                                    cursor_pos.x,
                                    cursor_pos.y,
                                );
                            }

                            let _ = commands::window_ops::reveal_main_window(app);
                        }
                        tauri_plugin_global_shortcut::ShortcutState::Released => {}
                    }
                }
            })
            .build(),
    );

    builder
        .invoke_handler(tauri::generate_handler![
            start_multi_drag,
            start_text_drag,
            open_popup_window,
            open_consent_window,
            add_files,
            save_pasted_data_base64,
            remove_files,
            get_files,
            get_file_icon_base64,
            clear_files,
            refresh_file_list,
            get_config,
            save_config,
            open_settings_window,
            set_autostart,
            register_hotkey,
            is_hotkey_active,
            accept_analytics_consent,
            decline_analytics_consent,
            check_config_exists,
            mark_drop_received,
        ])
        .setup(|app| {
            // Load configuration once. Saving creates the config directory.
            let config = config::AppConfig::load(app.handle());
            app.manage(Arc::new(Mutex::new(config.clone())));

            // Store the analytics service with the saved consent right away so
            // runtime consent changes always act on the live state. The
            // PostHog client is created lazily, only when consent is given.
            let analytics_state: analytics::AnalyticsState = Arc::new(Mutex::new(
                AnalyticsService::new(config.analytics_enabled, config.analytics_uuid.clone()),
            ));
            app.manage(analytics_state.clone());

            if config.analytics_enabled {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = analytics::ensure_client(&analytics_state).await {
                        warn!("Failed to initialize analytics: {}", e);
                    }
                    let _ = analytics::send_analytics_event(&app_handle, "app_started", None).await;
                });
            }

            // Create file list here
            let file_list: FileList = Arc::new(Mutex::new(Vec::new()));
            app.manage(file_list.clone());

            // Create drag state
            let drag_state = Arc::new(DragState {
                drag_started: AtomicBool::new(false),
                successful_drop: AtomicBool::new(false),
            });
            app.manage(drag_state.clone());

            // Check for updates on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(updater) = app_handle.updater() {
                    let update_available = matches!(updater.check().await, Ok(Some(_)));
                    let _ = analytics::send_analytics_event(
                        &app_handle,
                        "update_checked",
                        Some(vec![(
                            "update_available",
                            serde_json::Value::Bool(update_available),
                        )]),
                    )
                    .await;
                    // The updater window asks before installing anything.
                    if update_available {
                        if let Err(e) = commands::window_ops::open_updater_window(&app_handle) {
                            error!("Failed to open updater window: {}", e);
                        }
                    }
                }
            });

            // Register hotkey if configured
            if !config.hotkey.is_empty() {
                info!("Registering startup hotkey");
                if let Err(e) = register_hotkey(app.handle().clone(), config.hotkey.clone()) {
                    warn!("Failed to register startup hotkey: {}", e);
                } else {
                    info!("Startup hotkey registered successfully");
                }
            }

            #[cfg(desktop)]
            {
                let handle = app.handle();
                tray::create_tray(handle)?;
            }

            mouse_monitor::start_mouse_monitor(
                config.mouse_monitor.clone(),
                app.handle().clone(),
                drag_state.clone(),
            );

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            if let Some(window) = app.get_webview_window("main") {
                if is_autostart {
                    let _ = window.hide();
                }

                #[cfg(target_os = "windows")]
                if let Ok(hwnd_ptr) = window.hwnd() {
                    let hwnd = windows::Win32::Foundation::HWND(hwnd_ptr.0 as _);
                    drop_target::register_main_drop_target(app.handle(), hwnd);
                }
            }

            Ok(())
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::setup_logging();
    tracing::info!("Starting holdem application");

    build_app()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

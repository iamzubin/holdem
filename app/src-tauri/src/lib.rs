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
mod drop_report;
mod drop_target;

mod file_drop;
mod logging;
mod mouse_monitor;
mod presence;
mod shelf_store;
mod shelves;
mod preview;
mod image_tools;
mod item_actions;
mod window_style;
mod thumbnail;
#[cfg(desktop)]
mod tray;
mod utils;

use analytics::AnalyticsService;
use commands::{config_ops::*, drag_ops::*, file_ops::*, window_ops::*};
pub(crate) type FileList = Arc<Mutex<Vec<file::FileMetadata>>>;
/// Entries taken off the shelf by the last Remove or Clear, kept so the user
/// can undo it (see `restore_removed`).
///
/// A type of its own, never an alias: Tauri keeps app state by type, so an
/// alias of `FileList`'s type would make `app.manage` ignore it and hand out
/// the shelf list wherever the stash was asked for. Remove would then
/// overwrite the whole shelf with the removed items.
#[derive(Default)]
pub(crate) struct RemovedStash(Mutex<Vec<file::FileMetadata>>);

impl std::ops::Deref for RemovedStash {
    type Target = Mutex<Vec<file::FileMetadata>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Adds files passed on the command line to the shelf.
fn add_launch_paths(app: &tauri::AppHandle, args: &[String]) {
    let paths = commands::file_ops::launch_paths(args);
    if paths.is_empty() {
        return;
    }
    if let Some(file_list) = app.try_state::<FileList>() {
        file_drop::handle_file_drop_from_paths(paths, file_list.inner().clone(), app.clone());
    }
}

fn build_app() -> tauri::Builder<tauri::Wry> {
    let mut builder = tauri::Builder::default()
        // Must be registered first. Launching Holdem again (Start menu,
        // shortcut) shows the running shelf instead of starting a second
        // copy. The lock is per user session, so other Windows users on the
        // same machine can run their own instance.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            info!("Second launch detected, showing the existing shelf");
            // "Send to > Holdem" starts a second copy with the chosen paths.
            add_launch_paths(app, &args);
            if let Err(e) = commands::window_ops::reveal_main_window(app) {
                warn!("Failed to show the existing shelf: {}", e);
            }
        }))
        .plugin(tauri_plugin_process::init())
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
                            // The hotkey toggles: pressed again, it hides the shelf.
                            if window.is_visible().unwrap_or(false)
                                && window.is_focused().unwrap_or(false)
                            {
                                let _ = window.hide();
                                return;
                            }
                            let mut cursor_pos = POINT { x: 0, y: 0 };
                            if unsafe { GetCursorPos(&mut cursor_pos) }.is_ok() {
                                crate::utils::move_window_to_cursor(
                                    &window,
                                    cursor_pos.x,
                                    cursor_pos.y,
                                );
                            }

                            let _ = commands::window_ops::reveal_main_window(app);
                            analytics::track(app, "shelf_shown", vec![("via", "hotkey".into())]);
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
            start_note_drag,
            open_popup_window,
            close_popup,
            drag_threshold,
            open_consent_window,
            add_files,
            save_pasted_data_base64,
            remove_files,
            get_files,
            get_file_icon_base64,
            clear_files,
            restore_removed,
            paste_clipboard,
            copy_items_to_clipboard,
            shelves::get_shelves,
            shelves::create_shelf,
            shelves::rename_shelf,
            shelves::set_active_shelf,
            shelves::delete_shelf,
            shelves::reopen_shelf,
            shelves::move_to_shelf,
            preview::get_preview,
            item_actions::compress_items,
            image_tools::transform_images,
            item_actions::copy_items_to_folder,
            window_style::apply_window_style,
            set_pinned,
            rename_item,
            tray::set_tray_labels,
            open_shelf_item,
            reveal_shelf_item,
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
            // Start-on-login is registered with Windows only once the user
            // answers the first-run window (see persist_analytics_consent).
            let config = config::AppConfig::load(app.handle());
            // The shelf from the previous run, saved again on every change.
            let loaded = shelf_store::load(app.handle());
            if loaded.failed {
                file_drop::CLEANUP_BLOCKED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            let missing_on_restore = loaded.missing;
            app.manage::<shelves::Shelves>(Arc::new(Mutex::new(loaded.shelves)));
            let restored_files = loaded.files;
            let restored_count = restored_files.len();
            app.manage(Arc::new(Mutex::new(config.clone())));

            // Store the analytics service with the saved consent right away so
            // runtime consent changes always act on the live state. The
            // PostHog client is created lazily, only when consent is given.
            let analytics_state: analytics::AnalyticsState = Arc::new(Mutex::new(
                AnalyticsService::new(
                    config.analytics_enabled,
                    config.analytics_uuid.clone(),
                    app.package_info().version.to_string(),
                ),
            ));
            app.manage(analytics_state.clone());

            if config.analytics_enabled {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = analytics::ensure_client(&analytics_state).await {
                        warn!("Failed to initialize analytics: {}", e);
                    }
                    let _ = analytics::send_analytics_event(
                        &app_handle,
                        "app_started",
                        Some(vec![("restored_items", restored_count.into())]),
                    )
                    .await;
                });
            }

            let file_list: FileList = Arc::new(Mutex::new(restored_files));
            app.manage(file_list);
            app.manage(RemovedStash::default());
            shelf_store::save_on_change(app.handle());
            let elevated = mouse_monitor::holdem_is_elevated();
            if missing_on_restore > 0 || elevated {
                // Give the shelf page time to start listening.
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    commands::file_ops::notify_missing(&handle, missing_on_restore);
                    if elevated {
                        // Windows would silently refuse every drop from Explorer.
                        warn!("Holdem is running as administrator; drops from other apps are blocked");
                        drop_report::DropReport::single(drop_report::DropIssue::HoldemElevated)
                            .send(&handle);
                    }
                });
            }

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

            let launch_args: Vec<String> = std::env::args().collect();
            add_launch_paths(app.handle(), &launch_args);
            let is_autostart = launch_args.iter().any(|arg| arg == "--autostart");
            if let Some(window) = app.get_webview_window("main") {
                if is_autostart {
                    let _ = window.hide();
                }

                #[cfg(target_os = "windows")]
                if let Ok(hwnd_ptr) = window.hwnd() {
                    let hwnd = windows::Win32::Foundation::HWND(hwnd_ptr.0 as _);
                    drop_target::register_drop_target(app.handle(), "main", hwnd);
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

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    /// Tauri keeps state by type: if these were the same type, the stash
    /// would be the shelf list itself (see `RemovedStash`).
    #[test]
    fn every_managed_state_has_its_own_type() {
        assert_ne!(TypeId::of::<super::FileList>(), TypeId::of::<super::RemovedStash>());
    }
}

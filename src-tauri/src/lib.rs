use commands::{
    add_files, close_popup_window, get_files, open_popup_window, remove_file, rename_file,
    start_drag, start_multi_drag,
};
use file::FileMetadata;
use mouse_monitor::start_mouse_monitor;
use mouse_monitor::MouseMonitorConfig;
use std::sync::Arc;
use std::sync::Mutex;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod commands;
#[cfg(desktop)]
mod tray;

mod file; // Make sure to include the file module
mod mouse_monitor;
type FileList = Arc<Mutex<Vec<FileMetadata>>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let file_list: FileList = Arc::new(Mutex::new(Vec::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(file_list.clone())
        .invoke_handler(tauri::generate_handler![
            start_drag,
            start_multi_drag,
            open_popup_window,
            close_popup_window,
            add_files,
            remove_file,
            get_files,
            rename_file
        ])
        .setup(|app| {
            #[cfg(all(desktop))]
            {
                let handle = app.handle();
                tray::create_tray(handle)?;
            }

            // Start the mouse monitor with custom configuration
            let app_handle = app.handle().clone();
            let config = MouseMonitorConfig {
                required_shakes: 5,
                shake_time_limit: 1500,
                shake_threshold: 100,
                window_close_delay: 3000,
            };
            start_mouse_monitor(config, app_handle.clone());

            // #[cfg(target_os = "macos")]
            // apply_vibrancy(
            //     &app_handle.get_webview_window("main").unwrap(),
            //     NSVisualEffectMaterial::HudWindow,
            //     None,
            //     None,
            // )
            // .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            // #[cfg(target_os = "windows")]
            // apply_acrylic(
            //     &app_handle.get_webview_window("main").unwrap(),
            //     Some((106, 223, 0, 100)),
            // )
            // .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            // #[cfg(target_os = "windows")]
            // apply_acrylic(
            //     &app.get_webview_window("popup").unwrap(),
            //     Some((106, 223, 0, 100)),
            // )
            // .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

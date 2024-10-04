// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod file_handler;
#[cfg(desktop)]
mod tray;

use tauri::Manager; // Import WindowUrl
use tauri::WindowEvent;
use tempfile::TempDir; // Add this import

mod mouse_monitor;

use file_handler::FileStore;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(FileStore(Mutex::new(Vec::new())))
        .invoke_handler(tauri::generate_handler![
            file_handler::handle_file_drop,
            file_handler::get_stored_files,
            file_handler::remove_file,
            file_handler::rename_file,
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
        .invoke_handler(tauri::generate_handler![
            file_handler::handle_file_drop,
            file_handler::get_stored_files,
            file_handler::remove_file,
            file_handler::rename_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

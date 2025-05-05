use commands::{
    add_files, clear_files, close_popup_window, get_file_icon_base64, get_files, open_popup_window,
    remove_files, rename_file, start_drag, start_multi_drag, refresh_file_list,
};
use file::FileMetadata;
use mouse_monitor::start_mouse_monitor;
use mouse_monitor::MouseMonitorConfig;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Listener;
use tauri::Manager;
use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod commands;
#[cfg(desktop)]
mod tray;

mod file; // Make sure to include the file module
mod mouse_monitor;
type FileList = Arc<Mutex<Vec<FileMetadata>>>;

#[derive(Default)]
struct UpdaterState(Arc<Mutex<Option<tauri_plugin_updater::Update>>>);

fn handle_file_drop(event: tauri::Event, file_list: FileList, app_handle: tauri::AppHandle) {
    let payload: serde_json::Value = serde_json::from_str(event.payload()).unwrap_or_default();
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
                    .plugin(tauri_plugin_fs::init())
                    .plugin(tauri_plugin_shell::init())
                    .plugin(tauri_plugin_updater::Builder::new().build())
                    .manage(UpdaterState::default())
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
                    ])
                    .setup(|app| {
                        // Create file list here
                        let file_list: FileList = Arc::new(Mutex::new(Vec::new()));
                        app.manage(file_list.clone());

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

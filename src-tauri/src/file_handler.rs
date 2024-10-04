use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Event;
use tauri::{Emitter, State};
use tauri::{Listener, Manager};

#[derive(Clone, serde::Serialize, Debug)]
pub struct FileInfo {
    pub id: u64,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
}

pub struct FileStore(pub Mutex<Vec<FileInfo>>);

pub fn setup_drag_drop_listener(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let file_store = app.state::<FileStore>();

    let app_handle_clone = app_handle.clone();

    app.listen("tauri://file-drop", move |event| {
        println!("Drag event received {:?}", event);
        // let payload = event.payload();
        // let paths: Vec<String> = serde_json::from_str(payload).unwrap_or_default();
        // let new_files = handle_file_drop_internal(app_handle.clone(), paths, file_store);
        // if let Ok(files) = new_files {
        //     app_handle_clone.emit("files_dropped", files).unwrap();
        // }
    });
    Ok(())
}

fn handle_file_drop_internal(
    app: tauri::AppHandle,
    paths: Vec<String>,
    file_store: State<'_, FileStore>,
) -> Result<Vec<FileInfo>, String> {
    let mut stored_files = file_store
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire the file store lock: {:?}", e))?;
    let mut new_files = Vec::new();

    for path in paths {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            let file_name = path
                .file_name()
                .ok_or_else(|| "Failed to get file name".to_string())?
                .to_string_lossy()
                .into_owned();
            let file_info = FileInfo {
                id: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_millis() as u64,
                path: path.to_string_lossy().into_owned(),
                name: file_name,
                size: metadata.len(),
                is_directory: metadata.is_dir(),
            };
            new_files.push(file_info.clone());
            stored_files.push(file_info);
        }
    }

    app.emit("files_dropped", ()).map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
    }

    println!("Stored files: {:?}", stored_files);

    Ok(new_files)
}

#[tauri::command]
pub async fn get_stored_files(file_store: State<'_, FileStore>) -> Result<Vec<FileInfo>, String> {
    let stored_files = file_store.0.lock().unwrap();
    Ok(stored_files.clone())
}

#[tauri::command]
pub async fn remove_file(id: u64, file_store: State<'_, FileStore>) -> Result<(), String> {
    let mut stored_files = file_store.0.lock().unwrap();
    stored_files.retain(|file| file.id != id);
    Ok(())
}

#[tauri::command]
pub async fn rename_file(
    id: u64,
    new_name: String,
    file_store: State<'_, FileStore>,
) -> Result<(), String> {
    let mut stored_files = file_store.0.lock().unwrap();
    if let Some(file) = stored_files.iter_mut().find(|f| f.id == id) {
        file.name = new_name;
    }
    Ok(())
}

#[tauri::command]
pub async fn start_multi_drag(
    app: tauri::AppHandle,
    file_ids: Vec<u64>,
    file_store: State<'_, FileStore>,
) -> Result<(), String> {
    println!("Starting multi-file drag for: {:?}", file_ids);

    let stored_files = file_store
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire the file store lock: {:?}", e))?;
    // Remove the debug print statement that was causing the issue

    // pring stored_files
    println!("Stored files: {:?}", stored_files);

    let files_to_drag: Vec<std::path::PathBuf> = stored_files
        .iter()
        .filter(|file| file_ids.contains(&file.id))
        .map(|file| std::path::PathBuf::from(&file.path))
        .collect();

    println!("Starting multi-file drag for: {:?}", files_to_drag);

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found.".to_string())?;
    let hwnd_isize = window.hwnd().unwrap().0 as *mut c_void as isize;

    let item = drag::DragItem::Files(files_to_drag);

    // Define the on_drop_callback function
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Drag result: {:?}", result);
    };

    // Start the drag operation
    drag::start_drag(
        &hwnd_isize,
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .map_err(|e| format!("Failed to start drag operation: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn handle_file_drop(
    app: tauri::AppHandle,
    paths: Vec<String>,
    file_store: State<'_, FileStore>,
) -> Result<Vec<FileInfo>, String> {
    handle_file_drop_internal(app, paths, file_store)
}

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tauri::{Emitter, Manager, State};

#[derive(Clone, serde::Serialize)]
pub struct FileInfo {
    pub id: u64,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
}

pub struct FileStore(pub Mutex<Vec<FileInfo>>);

#[tauri::command]
pub async fn handle_file_drop(
    app: tauri::AppHandle,
    paths: Vec<String>,
    file_store: State<'_, FileStore>,
) -> Result<Vec<FileInfo>, String> {
    let mut stored_files = file_store.0.lock().unwrap();
    let mut new_files = Vec::new();

    for path in paths {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            let file_info = FileInfo {
                id: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                path: path.to_string_lossy().into_owned(),
                name: path.file_name().unwrap().to_string_lossy().into_owned(),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
            };
            new_files.push(file_info.clone());
            stored_files.push(file_info);
        }
    }

    app.emit("files_dropped", ()).map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
    }

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
    let stored_files = file_store.0.lock().unwrap();
    let files_to_drag: Vec<String> = stored_files
        .iter()
        .filter(|file| file_ids.contains(&file.id))
        .map(|file| file.path.clone())
        .collect();

    // Here you would typically interact with the OS to start the drag operation
    // For now, we'll just log the files being dragged
    println!("Starting multi-file drag for: {:?}", files_to_drag);

    // Emit an event to notify the frontend that the drag has started
    app.emit("multi_drag_started", files_to_drag)
        .map_err(|e| e.to_string())?;

    Ok(())
}

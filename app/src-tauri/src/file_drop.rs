use std::path::PathBuf;
use tauri::{AppHandle, Event, Emitter};
use crate::file::{get_dir_size, FileMetadata};
use crate::FileList;

pub fn handle_file_drop(event: Event, file_list: FileList, app_handle: AppHandle) {
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
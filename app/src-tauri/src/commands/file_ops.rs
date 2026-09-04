use crate::analytics;
use crate::file::{get_dir_size, FileMetadata};
use crate::thumbnail::get_thumbnail_base64;
use crate::{DragState, FileList};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn add_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    drag_state: State<'_, Arc<DragState>>,
    files: Vec<String>,
) -> Result<(), String> {
    if !files.is_empty() {
        // Mark the drop before metadata work so shake-to-show does not close the window.
        drag_state.successful_drop.store(true, Ordering::Relaxed);
        drag_state.drag_started.store(false, Ordering::Relaxed);
    }

    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    for path_str in files.iter() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let metadata = path.metadata().map_err(|e| e.to_string())?;

            // Calculate size correctly for directories
            let size = if metadata.is_dir() {
                get_dir_size(&path).unwrap_or(0)
            } else {
                metadata.len()
            };

            let file = FileMetadata {
                id: list.len() as u64,
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                path: path.clone(),
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
                list.push(file);
            }
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    if sanitized.trim().is_empty() {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

#[tauri::command]
pub fn save_pasted_text(
    text: String,
    extension: String,
    original_name: Option<String>,
) -> Result<String, String> {
    use std::io::Write;
    let timestamp = chrono::Local::now();
    let folder_name = timestamp.format("%Y%m%d").to_string();
    let drop_folder = std::env::temp_dir().join("holdem_drops").join(&folder_name);
    if let Err(e) = std::fs::create_dir_all(&drop_folder) {
        return Err(format!("Failed to create drop folder: {}", e));
    }

    let file_name = if let Some(name) = original_name.as_deref().filter(|n| !n.trim().is_empty()) {
        sanitize_filename(name)
    } else {
        format!("note_{}.{}", timestamp.format("%H%M%S"), extension)
    };
    let new_path = drop_folder.join(&file_name);

    let mut file = std::fs::File::create(&new_path).map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_pasted_data_base64(
    data_base64: String,
    extension: String,
    original_name: Option<String>,
) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::io::Write;

    let bytes = general_purpose::STANDARD
        .decode(data_base64)
        .map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now();
    let folder_name = timestamp.format("%Y%m%d").to_string();
    let drop_folder = std::env::temp_dir().join("holdem_drops").join(&folder_name);
    if let Err(e) = std::fs::create_dir_all(&drop_folder) {
        return Err(format!("Failed to create drop folder: {}", e));
    }

    let file_name = if let Some(name) = original_name.as_deref().filter(|n| !n.trim().is_empty()) {
        sanitize_filename(name)
    } else {
        format!("drop_{}.{}", timestamp.format("%H%M%S"), extension)
    };
    let new_path = drop_folder.join(&file_name);

    let mut file = std::fs::File::create(&new_path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn download_image_to_shelf(url: String) -> Result<String, String> {
    use std::io::Write;
    let timestamp = chrono::Local::now();
    let folder_name = timestamp.format("%Y%m%d").to_string();
    let drop_folder = std::env::temp_dir().join("holdem_drops").join(&folder_name);
    if let Err(e) = std::fs::create_dir_all(&drop_folder) {
        return Err(format!("Failed to create drop folder: {}", e));
    }

    let (file_name, ext) = if let Ok(parsed_url) = url::Url::parse(&url) {
        let path_segment = parsed_url
            .path_segments()
            .and_then(|mut segs| segs.next_back())
            .unwrap_or("");
        
        let path_obj = std::path::Path::new(path_segment);
        let extracted_ext = path_obj
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !path_segment.is_empty() && !extracted_ext.is_empty() {
            (sanitize_filename(path_segment), extracted_ext.to_string())
        } else {
            let ext = if !extracted_ext.is_empty() { extracted_ext } else { "png" };
            (format!("download_{}.{}", timestamp.format("%H%M%S"), ext), ext.to_string())
        }
    } else {
        ("download_image.png".to_string(), "png".to_string())
    };

    let mut new_path = drop_folder.join(&file_name);
    if new_path.exists() {
        let stem = std::path::Path::new(&file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("download");
        new_path = drop_folder.join(format!("{}_{}.{}", stem, timestamp.format("%H%M%S"), ext));
    }

    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let mut file = std::fs::File::create(&new_path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}
#[tauri::command]
pub fn remove_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    file_ids: Vec<u64>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    let mut removed_files = Vec::new();
    for file_id in file_ids {
        if let Some(pos) = list.iter().position(|f| f.id == file_id) {
            let file_name = list[pos].name.clone();
            list.remove(pos);
            removed_files.push(file_name);
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        } else {
            return Err(format!("File with ID {} not found", file_id));
        }
    }

    // Send analytics events for removed files (fire and forget)
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        for _ in removed_files {
            let _ = analytics::send_file_removed_event(&app_handle_clone).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_files(file_list: State<'_, FileList>) -> Result<Vec<FileMetadata>, String> {
    let list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    Ok(list.clone())
}

#[tauri::command]
pub fn rename_file(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    file_id: u64,
    new_name: String,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    if let Some(file) = list.iter_mut().find(|f| f.id == file_id) {
        file.name = new_name.clone();

        // Send analytics event for file rename (fire and forget)
        let app_handle_clone = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let _ = analytics::send_file_renamed_event(&app_handle_clone).await;
        });

        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("File not found".to_string())
    }
}

#[tauri::command]
pub fn clear_files(app_handle: AppHandle, file_list: State<'_, FileList>) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    let num_files = list.len();
    list.clear();

    // Send analytics event for clearing files (fire and forget)
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_files_cleared_event(&app_handle_clone, num_files).await;
    });

    app_handle
        .emit("files_updated", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn refresh_file_list(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    let mut needs_update = false;

    // Create a new list to store valid files
    let mut new_list = Vec::new();

    for file in list.iter() {
        if file.path.exists() {
            new_list.push(file.clone());
        } else {
            needs_update = true;
        }
    }

    if needs_update {
        *list = new_list;
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_file_icon_base64(
    _app_handle: AppHandle,
    _file_list: State<'_, FileList>,
    file_path: &str,
) -> Result<String, String> {
    let file_path = file_path.to_string();
    get_thumbnail_base64(&file_path).map_err(|e| e.to_string())
}

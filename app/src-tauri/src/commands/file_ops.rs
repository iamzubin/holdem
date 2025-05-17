use std::path::PathBuf;
use tauri::{AppHandle, State, Emitter};
use crate::file::{get_dir_size, FileMetadata};
use crate::FileList;

#[tauri::command]
pub fn add_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    files: Vec<String>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    for (_index, path_str) in files.iter().enumerate() {
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

#[tauri::command]
pub fn remove_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    file_ids: Vec<u64>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    for file_id in file_ids {
        if let Some(pos) = list.iter().position(|f| f.id == file_id) {
            list.remove(pos);
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        } else {
            return Err(format!("File with ID {} not found", file_id));
        }
    }
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
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err(format!("File with ID {} not found", file_id))
    }
}

#[tauri::command]
pub fn clear_files(
    app_handle: AppHandle,
    _file_list: State<'_, FileList>,
) -> Result<(), String> {
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
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    file_path: &str,
) -> Result<String, String> {
    let file_path = file_path.to_string();
    if !std::path::Path::new(&file_path).exists() {
        // Remove the file from the list if it no longer exists
        let mut list = file_list
            .lock()
            .map_err(|_| "Failed to acquire lock".to_string())?;
        if let Some(pos) = list
            .iter()
            .position(|f| f.path.to_string_lossy() == file_path)
        {
            list.remove(pos);
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        }
        return Ok("".to_string()); // Return empty string for non-existent files
    }
    Ok(windows_icons::get_icon_base64_by_path(&file_path))
} 